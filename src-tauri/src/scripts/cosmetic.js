(function () {
  if (window.__starCosmetic) return;
  window.__starCosmetic = true;

  var SELECTORS = [
    '#google_image_div',
    'ins.adsbygoogle',
    'iframe[id^="google_ads_iframe"]',
    'iframe[id^="aswift_"]',
    'iframe[src*="doubleclick.net"]',
    'iframe[src*="googlesyndication.com"]',
    'iframe[src*="amazon-adsystem.com"]',
    'iframe[src*="adnxs.com"]',
    'iframe[src*="criteo"]',
    'div[id^="div-gpt-ad"]',
    'div[id^="google_ads_"]',
    'div[id^="taboola-"]',
    'div[class^="taboola"]',
    'div[id^="outbrain_widget"]',
    'div[class*="OUTBRAIN"]',
    'div[data-ad-client]',
    'div[data-ad-slot]',
    'div[data-adunit]',
    'div[data-google-query-id]',
    '[id^="ad-container"]',
    '[id^="ad-slot"]',
    '[id^="banner-ad"]',
    '[class^="ad-slot"]',
    '[class*="advertisement"]',
    '[class*="sponsored-post"]',
    '[aria-label="advertisement" i]',
    '[aria-label="Advertisement" i]',
    '[id*="banner_ad" i]',
    '[class*="banner-ad" i]',
    '[class*="ad-banner" i]',
    '[id*="adbanner" i]',
    '[id^="adv_"]',
    '[id^="adv-"]',
    '[class^="adv-"]',
    'img[src*="/ads/"]',
    'img[src*="/adv/"]',
    'img[src*="/banners/"]',
    'img[src*="468x60"]',
    'img[src*="728x90"]',
    'img[src*="300x250"]',
    'img[src*="336x280"]',
    'img[src*="160x600"]',
    'img[src*="320x50"]',
    'object[data*="/ads/"]',
    'embed[src*="/ads/"]',
    'object[type="application/x-shockwave-flash"]',
    'a[href*="/adclick"]',
    'a[href*="doubleclick.net"]'
  ];

  var style = document.createElement('style');
  style.id = 'star-cosmetic';
  style.textContent = SELECTORS.join(',') + '{display:none!important}';

  function attach() {
    var head = document.head || document.documentElement;
    if (head && !document.getElementById('star-cosmetic')) head.appendChild(style);
  }

  attach();
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', attach, { once: true });
  }

  var LABELS = /^(advertisement|advertisements|sponsored|pubblicità|anuncio|werbung|publicité)$/i;

  function isAdLabel(el) {
    if (!el || el.childElementCount > 0) return false;
    var text = (el.textContent || '').trim();
    return text.length > 0 && text.length < 24 && LABELS.test(text);
  }

  function sweep(root) {
    var nodes = root.querySelectorAll('span,div,p,h2,h3,h4');
    for (var i = 0; i < nodes.length && i < 400; i++) {
      var node = nodes[i];
      if (!isAdLabel(node)) continue;
      var box = node.parentElement;
      if (!box || box === document.body) continue;
      if (box.getBoundingClientRect().height > 700) continue;
      box.style.setProperty('display', 'none', 'important');
    }
  }

  var pending = false;
  function schedule() {
    if (pending) return;
    pending = true;
    requestAnimationFrame(function () {
      pending = false;
      try {
        sweep(document);
      } catch (e) {}
    });
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', schedule, { once: true });
  } else {
    schedule();
  }

  var observer = new MutationObserver(schedule);
  function observe() {
    if (document.body) observer.observe(document.body, { childList: true, subtree: true });
  }
  if (document.body) observe();
  else document.addEventListener('DOMContentLoaded', observe, { once: true });

  setTimeout(function () {
    observer.disconnect();
  }, 20000);
})();
