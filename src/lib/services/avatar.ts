const AVATAR_SIZE = 256;
const AVATAR_QUALITY = 0.85;

export function cropAvatar(file: File): Promise<string | null> {
    if (!file.type.startsWith('image/')) return Promise.resolve(null);

    return new Promise((resolve) => {
        const reader = new FileReader();
        reader.onerror = () => resolve(null);
        reader.onload = () => {
            const img = new Image();
            img.onerror = () => resolve(null);
            img.onload = () => {
                const canvas = document.createElement('canvas');
                canvas.width = canvas.height = AVATAR_SIZE;
                const ctx = canvas.getContext('2d');
                if (!ctx) return resolve(null);

                const edge = Math.min(img.width, img.height);
                ctx.drawImage(
                    img,
                    (img.width - edge) / 2,
                    (img.height - edge) / 2,
                    edge,
                    edge,
                    0,
                    0,
                    AVATAR_SIZE,
                    AVATAR_SIZE
                );
                resolve(canvas.toDataURL('image/jpeg', AVATAR_QUALITY));
            };
            img.src = String(reader.result);
        };
        reader.readAsDataURL(file);
    });
}
