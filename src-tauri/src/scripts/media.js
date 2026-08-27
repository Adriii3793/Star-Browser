(function () {
    if (window.__starMedia) return;

    var AUDIO_PREFIX = '@@star-audio@@:';
    var CHANNEL = '__star_media__';
    var SWEEP_MS = 1000;

    var SITE_CONTROLS = [
        '[data-testid="control-button-playpause"]', // Spotify
        '[data-testid="play-pause-button"]',
        '.ytp-play-button', // YouTube
        'tp-yt-paper-icon-button.play-pause-button', // YouTube Music
        '.playControl', // SoundCloud
        'button.playback-play', // Apple Music
        '.player-controls__buttons button[aria-label*="lay"]'
    ];
    var GENERIC_CONTROLS = [
        'button[aria-label="Pause"]',
        'button[aria-label="Play"]',
        'button[title="Pause"]',
        'button[title="Play"]'
    ];

    var muted = false;
    var localAudible = false;
    var frames = []; // [{ src: Window, audible: boolean }] — top frame only
    var seen = []; // media elements we have already tracked
    var lastSignal = null;

    var titleGen = 0;
    var titleBefore = null;

    function signalViaTitle(signal) {
        if (titleBefore === null) titleBefore = document.title;
        var gen = ++titleGen;
        document.title = signal;
        setTimeout(function () {
            if (gen !== titleGen) return;
            var original = titleBefore;
            titleBefore = null;
            document.title = original;
        }, 0);
    }

    function anyFrameAudible() {
        for (var i = 0; i < frames.length; i++) {
            if (frames[i].audible) return true;
        }
        return false;
    }

    function publish() {
        if (window.top !== window) {
            try {
                window.top.postMessage({ channel: CHANNEL, audible: localAudible }, '*');
            } catch (e) {}
            return;
        }
        var payload = (localAudible || anyFrameAudible() ? '1' : '0') + ',' + (muted ? '1' : '0');
        if (payload === lastSignal) return;
        lastSignal = payload;
        signalViaTitle(AUDIO_PREFIX + payload);
    }

    function isMedia(el) {
        return !!el && (el.tagName === 'VIDEO' || el.tagName === 'AUDIO');
    }

    function track(el) {
        if (!isMedia(el) || seen.indexOf(el) !== -1) return;
        seen.push(el);
        if (muted) {
            try {
                el.muted = true;
            } catch (e) {}
        }
    }

    function playing(el) {
        try {
            return !el.paused && !el.ended && el.readyState >= 2;
        } catch (e) {
            return false;
        }
    }

    function recompute() {
        var audible = false;
        for (var i = seen.length - 1; i >= 0; i--) {
            var el = seen[i];
            if (el.isConnected === false) {
                seen.splice(i, 1);
                continue;
            }
            if (playing(el)) audible = true;
        }
        if (audible !== localAudible) {
            localAudible = audible;
            publish();
        }
    }

    function sweep() {
        try {
            var list = document.querySelectorAll('video,audio');
            for (var i = 0; i < list.length; i++) track(list[i]);
        } catch (e) {}
        recompute();
    }

    var EVENTS = ['play', 'playing', 'pause', 'ended', 'emptied', 'volumechange', 'loadeddata'];
    for (var e = 0; e < EVENTS.length; e++) {
        document.addEventListener(
            EVENTS[e],
            function (event) {
                if (!isMedia(event.target)) return;
                track(event.target);
                recompute();
            },
            true
        );
    }

    setInterval(sweep, SWEEP_MS);
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', sweep, { once: true });
    } else {
        sweep();
    }

    function visible(el) {
        if (!el) return false;
        try {
            var rect = el.getBoundingClientRect();
            return rect.width > 0 && rect.height > 0;
        } catch (err) {
            return false;
        }
    }

    function clickFirst(selectors) {
        for (var i = 0; i < selectors.length; i++) {
            var el = null;
            try {
                el = document.querySelector(selectors[i]);
            } catch (err) {}
            if (visible(el)) {
                el.click();
                return true;
            }
        }
        return false;
    }

    function broadcast(cmd, value) {
        var iframes;
        try {
            iframes = document.querySelectorAll('iframe');
        } catch (err) {
            return;
        }
        for (var i = 0; i < iframes.length; i++) {
            try {
                iframes[i].contentWindow.postMessage(
                    { channel: CHANNEL, cmd: cmd, value: value },
                    '*'
                );
            } catch (err) {}
        }
    }

    function applyMuted(next) {
        muted = !!next;
        sweep();
        for (var i = 0; i < seen.length; i++) {
            try {
                seen[i].muted = muted;
            } catch (err) {}
        }
        lastSignal = null; // force the mute flag out even if audibility did not change
        publish();
        broadcast('mute', muted);
    }

    function toggle() {
        if (clickFirst(SITE_CONTROLS)) {
            broadcast('sync');
            return;
        }
        sweep();

        var active = [];
        for (var i = 0; i < seen.length; i++) {
            if (playing(seen[i])) active.push(seen[i]);
        }
        if (active.length) {
            for (var p = 0; p < active.length; p++) {
                try {
                    active[p].pause();
                } catch (err) {}
            }
            recompute();
            broadcast('toggle');
            return;
        }

        var target = null;
        for (var c = 0; c < seen.length; c++) {
            var el = seen[c];
            if (el.currentTime > 0 || el.duration > 0) {
                target = el;
                break;
            }
        }
        if (!target) target = seen[0];

        if (target) {
            try {
                var started = target.play();
                if (started && typeof started.catch === 'function') {
                    started.catch(function () {
                        clickFirst(GENERIC_CONTROLS);
                    });
                }
                return;
            } catch (err) {}
        }

        if (!clickFirst(GENERIC_CONTROLS)) broadcast('toggle');
    }

    function stop() {
        sweep();
        for (var i = 0; i < seen.length; i++) {
            var el = seen[i];
            try {
                el.pause();
            } catch (err) {}
            try {
                var stream = el.srcObject;
                if (stream && typeof stream.getTracks === 'function') {
                    stream.getTracks().forEach(function (t) {
                        t.stop();
                    });
                }
            } catch (err) {}
        }
        recompute();
        broadcast('stop');
    }

    window.addEventListener('message', function (event) {
        var data = event.data;
        if (!data || data.channel !== CHANNEL) return;

        if (typeof data.audible === 'boolean') {
            for (var i = 0; i < frames.length; i++) {
                if (frames[i].src === event.source) {
                    frames[i].audible = data.audible;
                    publish();
                    return;
                }
            }
            frames.push({ src: event.source, audible: data.audible });
            publish();
            return;
        }

        if (data.cmd === 'mute') applyMuted(data.value);
        else if (data.cmd === 'toggle') toggle();
        else if (data.cmd === 'stop') stop();
        else if (data.cmd === 'sync') sweep();
    });

    window.__starMedia = {
        setMuted: applyMuted,
        toggle: toggle,
        stop: stop,
        sweep: sweep
    };
})();
