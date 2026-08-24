(function () {
    if (window.__starOffline) return;
    window.__starOffline = true;

    var BANNER_ID = "__star-offline-banner";

    function banner() {
        var el = document.getElementById(BANNER_ID);
        if (el) return el;

        var dark = window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches;
        var page = dark ? "#1c1917" : "#ffffff";
        var ink = dark ? "#f2efec" : "#1f2328";
        var muted = dark ? "#a5a09b" : "#5c636a";
        var line = dark ? "rgba(255,255,255,.16)" : "rgba(31,35,40,.14)";

        el = document.createElement("div");
        el.id = BANNER_ID;
        el.setAttribute(
            "style",
            "position:fixed;left:50%;top:16px;transform:translateX(-50%);z-index:2147483647;" +
            "display:flex;align-items:center;gap:16px;max-width:calc(100vw - 32px);" +
            "padding:11px 12px 11px 18px;border:1px solid " + line + ";border-radius:10px;" +
            "background:" + page + ";color:" + ink + ";" +
            "font:500 13px/1.4 -apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;" +
            "box-shadow:0 8px 28px rgba(0,0,0,.18)"
        );

        var text = document.createElement("span");
        text.textContent = "No internet connection";
        text.setAttribute("style", "white-space:nowrap");

        var sub = document.createElement("span");
        sub.textContent = "This page may be out of date.";
        sub.setAttribute("style", "color:" + muted + ";font-weight:400;white-space:nowrap");

        var retry = document.createElement("button");
        retry.type = "button";
        retry.textContent = "Reload";
        retry.setAttribute(
            "style",
            "padding:6px 14px;border:1px solid " + line + ";border-radius:7px;background:transparent;" +
            "color:inherit;font:inherit;font-size:12px;cursor:pointer"
        );
        retry.addEventListener("click", function () { location.reload(); });

        el.appendChild(text);
        el.appendChild(sub);
        el.appendChild(retry);
        (document.body || document.documentElement).appendChild(el);
        return el;
    }

    function show() { banner(); }

    function hide() {
        var el = document.getElementById(BANNER_ID);
        if (el && el.parentElement) el.parentElement.removeChild(el);
    }

    function render() {
        if (!navigator.onLine) show();
    }

    window.addEventListener("offline", show);
    window.addEventListener("online", function () {
        hide();
        location.reload();
    });

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", render, { once: true });
    } else {
        render();
    }
})();
