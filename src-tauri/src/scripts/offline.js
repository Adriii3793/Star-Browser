(function () { 
    var SHOWN = "__starOfflineShown";

    function render() {
        if (window[SHOWN]) return;
        window[SHOWN] = true;

        document.documentElement.innerHTML =
        '<head><meta charset="utf-8"><title>No connection</title></head>' +
        '<body style="margin:0;display:flex;align-items:center;justify-content:center;' +
        'min-height:100vh;background:#f7f1ec;color:#4a3a2e;' +
        "font-family:Inter,-apple-system,'Segoe UI',Roboto,sans-serif\">" +
        '<div style="text-align:center;max-width:420px;padding:40px 24px">' +
        '<div style="font-size:64px;margin-bottom:16px">🛰️</div>' +
        '<h1 style="margin:0 0 10px;font-size:26px;font-weight:600">You are offline</h1>' +
        '<p style="margin:0 0 28px;font-size:15px;line-height:1.6;color:#8a6b57">' +
        "Star can't reach the internet right now. Check your connection and try again.</p>" +
        '<button id="star-retry" style="padding:11px 26px;font:inherit;font-size:15px;' +
        "font-weight:500;background:#80a4d4;color:#fff;border:none;border-radius:8px;" +
        'cursor:pointer">Try again</button>' +
        "</div></body>";

        var retry = document.getElementById("star-retry");
        if (retry) retry.addEventListener("click", function () {location.reload();});
    }
    
    if (!navigator.onLine) render();
    window.addEventListener("offline", render);
    window.addEventListener("online", function () { location.reload();});
})();