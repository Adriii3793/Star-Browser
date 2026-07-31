import { call } from './ipc';

export interface SttStatus {
	available: boolean;
	model: string | null;
	reason: string | null;
}

export function sttStatus(): Promise<SttStatus> {
	return call('stt_status');
}

function transcribe(audioBase64: string, lang: string | null): Promise<string> {
	return call('stt_transcribe', { audioBase64, lang });
}

const TARGET_RATE = 16000;
const SILENCE_RMS = 0.012;
const MIN_SPEECH_MS = 350;
const MAX_TAKE_MS = 30000;

function downsample(input: Float32Array, from: number, to: number): Float32Array {
	if (to >= from) return input;
	const ratio = from / to;
	const out = new Float32Array(Math.floor(input.length / ratio));
	let pos = 0;
	for (let i = 0; i < out.length; i++) {
		const start = Math.floor(i * ratio);
		const end = Math.min(input.length, Math.floor((i + 1) * ratio));
		let sum = 0;
		for (let j = start; j < end; j++) sum += input[j];
		out[pos++] = end > start ? sum / (end - start) : 0;
	}
	return out;
}

function encodeWav(samples: Float32Array, rate: number): Blob {
	const buffer = new ArrayBuffer(44 + samples.length * 2);
	const view = new DataView(buffer);

	const writeText = (offset: number, value: string) => {
		for (let i = 0; i < value.length; i++) view.setUint8(offset + i, value.charCodeAt(i));
	};

	writeText(0, 'RIFF');
	view.setUint32(4, 36 + samples.length * 2, true);
	writeText(8, 'WAVE');
	writeText(12, 'fmt ');
	view.setUint32(16, 16, true);
	view.setUint16(20, 1, true);
	view.setUint16(22, 1, true);
	view.setUint32(24, rate, true);
	view.setUint32(28, rate * 2, true);
	view.setUint16(32, 2, true);
	view.setUint16(34, 16, true);
	writeText(36, 'data');
	view.setUint32(40, samples.length * 2, true);

	let offset = 44;
	for (let i = 0; i < samples.length; i++) {
		const clamped = Math.max(-1, Math.min(1, samples[i]));
		view.setInt16(offset, clamped < 0 ? clamped * 0x8000 : clamped * 0x7fff, true);
		offset += 2;
	}
	return new Blob([buffer], { type: 'audio/wav' });
}

function toBase64(blob: Blob): Promise<string> {
	return new Promise((resolve, reject) => {
		const reader = new FileReader();
		reader.onload = () => {
			const result = String(reader.result ?? '');
			resolve(result.slice(result.indexOf(',') + 1));
		};
		reader.onerror = () => reject(new Error('Could not read audio'));
		reader.readAsDataURL(blob);
	});
}

export interface RecorderHooks {
	onLevel?: (rms: number) => void;
	onEndOfSpeech?: () => void;
	autoStop?: boolean;
	silenceMs?: number;
}

export class VoiceRecorder {
	#stream: MediaStream | null = null;
	#context: AudioContext | null = null;
	#node: ScriptProcessorNode | null = null;
	#source: MediaStreamAudioSourceNode | null = null;
	#chunks: Float32Array[] = [];
	#rate = 48000;
	#running = false;
	#spokeMs = 0;
	#quietMs = 0;
	#elapsedMs = 0;
	#hooks: RecorderHooks = {};
	#ended = false;

	get active(): boolean {
		return this.#running;
	}

	async start(hooks: RecorderHooks = {}) {
		if (this.#running) return;
		this.#hooks = hooks;
		this.#chunks = [];
		this.#spokeMs = 0;
		this.#quietMs = 0;
		this.#elapsedMs = 0;
		this.#ended = false;

		this.#stream = await navigator.mediaDevices.getUserMedia({
			audio: {
				channelCount: 1,
				echoCancellation: true,
				noiseSuppression: true,
				autoGainControl: true
			}
		});

		this.#context = new AudioContext();
		this.#rate = this.#context.sampleRate;
		this.#source = this.#context.createMediaStreamSource(this.#stream);
		this.#node = this.#context.createScriptProcessor(4096, 1, 1);

		const silenceMs = hooks.silenceMs ?? 1200;

		this.#node.onaudioprocess = (event) => {
			if (!this.#running) return;
			const input = event.inputBuffer.getChannelData(0);
			this.#chunks.push(new Float32Array(input));

			let sum = 0;
			for (let i = 0; i < input.length; i++) sum += input[i] * input[i];
			const rms = Math.sqrt(sum / input.length);
			this.#hooks.onLevel?.(rms);

			const sliceMs = (input.length / this.#rate) * 1000;
			this.#elapsedMs += sliceMs;

			if (rms >= SILENCE_RMS) {
				this.#spokeMs += sliceMs;
				this.#quietMs = 0;
			} else if (this.#spokeMs >= MIN_SPEECH_MS) {
				this.#quietMs += sliceMs;
			}

			const done =
				this.#elapsedMs >= MAX_TAKE_MS ||
				(hooks.autoStop && this.#spokeMs >= MIN_SPEECH_MS && this.#quietMs >= silenceMs);

			if (done && !this.#ended) {
				this.#ended = true;
				this.#hooks.onEndOfSpeech?.();
			}
		};

		this.#source.connect(this.#node);
		this.#node.connect(this.#context.destination);
		this.#running = true;
	}

	#teardown() {
		this.#running = false;
		try {
			this.#node?.disconnect();
			this.#source?.disconnect();
		} catch {}
		if (this.#node) this.#node.onaudioprocess = null;
		this.#stream?.getTracks().forEach((t) => t.stop());
		void this.#context?.close().catch(() => {});
		this.#node = null;
		this.#source = null;
		this.#stream = null;
		this.#context = null;
	}

	#collect(): Float32Array {
		let total = 0;
		for (const c of this.#chunks) total += c.length;
		const merged = new Float32Array(total);
		let at = 0;
		for (const c of this.#chunks) {
			merged.set(c, at);
			at += c.length;
		}
		return merged;
	}

	cancel() {
		if (!this.#running && this.#chunks.length === 0) return;
		this.#chunks = [];
		this.#teardown();
	}

	async stop(lang: string | null): Promise<string> {
		if (!this.#running) return '';
		const rate = this.#rate;
		const spoke = this.#spokeMs;
		const merged = this.#collect();
		this.#chunks = [];
		this.#teardown();

		if (spoke < MIN_SPEECH_MS || merged.length === 0) return '';

		const wav = encodeWav(downsample(merged, rate, TARGET_RATE), TARGET_RATE);
		const base64 = await toBase64(wav);
		return transcribe(base64, lang);
	}
}

export function ttsSupported(): boolean {
	return typeof window !== 'undefined' && 'speechSynthesis' in window;
}

let voicesReady: Promise<SpeechSynthesisVoice[]> | null = null;

function loadVoices(): Promise<SpeechSynthesisVoice[]> {
	if (voicesReady) return voicesReady;
	voicesReady = new Promise((resolve) => {
		const existing = speechSynthesis.getVoices();
		if (existing.length > 0) {
			resolve(existing);
			return;
		}
		const done = () => {
			speechSynthesis.removeEventListener('voiceschanged', done);
			resolve(speechSynthesis.getVoices());
		};
		speechSynthesis.addEventListener('voiceschanged', done);
		setTimeout(done, 1500);
	});
	return voicesReady;
}

function scriptOf(text: string): string | null {
	if (/[一-鿿]/.test(text)) return 'zh';
	if (/[぀-ヿ]/.test(text)) return 'ja';
	if (/[가-힯]/.test(text)) return 'ko';
	if (/[؀-ۿ]/.test(text)) return 'ar';
	if (/[֐-׿]/.test(text)) return 'he';
	if (/[Ѐ-ӿ]/.test(text)) return 'ru';
	if (/[฀-๿]/.test(text)) return 'th';
	if (/[ऀ-ॿ]/.test(text)) return 'hi';
	if (/[Ͱ-Ͽ]/.test(text)) return 'el';
	return null;
}

async function pickVoice(lang: string | null): Promise<SpeechSynthesisVoice | null> {
	const voices = await loadVoices();
	if (voices.length === 0 || !lang) return null;
	const want = lang.toLowerCase().replace('_', '-');
	const base = want.split('-')[0];
	return (
		voices.find((v) => v.lang.toLowerCase().replace('_', '-') === want) ??
		voices.find((v) => v.lang.toLowerCase().startsWith(base)) ??
		null
	);
}

export function stopSpeaking() {
	if (!ttsSupported()) return;
	try {
		speechSynthesis.cancel();
	} catch {}
}

export async function speak(
	text: string,
	options: { lang?: string | null; rate?: number } = {}
): Promise<void> {
	if (!ttsSupported()) return;
	const clean = text
		.replace(/```[\s\S]*?```/g, ' ')
		.replace(/[*_`#>|]/g, '')
		.trim();
	if (!clean) return;

	stopSpeaking();

	const lang = options.lang ?? scriptOf(clean) ?? navigator.language ?? null;
	const voice = await pickVoice(lang);

	const parts = (clean.match(/[^.!?\n]+[.!?\n]*/g) ?? [clean])
		.map((c) => c.trim())
		.filter(Boolean)
		.reduce<string[]>((acc, part) => {
			const last = acc[acc.length - 1];
			if (last && last.length + part.length < 180) acc[acc.length - 1] = `${last} ${part}`;
			else acc.push(part);
			return acc;
		}, []);

	for (const part of parts) {
		await new Promise<void>((resolve) => {
			const utter = new SpeechSynthesisUtterance(part);
			if (voice) utter.voice = voice;
			if (lang) utter.lang = voice?.lang ?? lang;
			utter.rate = options.rate ?? 1;
			utter.onend = () => resolve();
			utter.onerror = () => resolve();
			speechSynthesis.speak(utter);
		});
	}
}
