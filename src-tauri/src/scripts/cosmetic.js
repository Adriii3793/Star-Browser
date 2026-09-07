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

  function hideBoxOf(node) {
    if (!isAdLabel(node)) return;
    var box = node.parentElement;
    if (!box || box === document.body) return;
    if (box.getBoundingClientRect().height > 700) return;
    box.style.setProperty('display', 'none', 'important');
  }

  function sweep(roots) {
    for (var r = 0; r < roots.length; r++) {
      var root = roots[r];
      if (root.nodeType === 1) hideBoxOf(root);
      var nodes;
      try {
        nodes = root.querySelectorAll('span,div,p,h2,h3,h4');
      } catch (e) {
        continue;
      }
      for (var i = 0; i < nodes.length && i < 400; i++) hideBoxOf(nodes[i]);
    }
  }

  var SWEEP_MS = 250;
  var MAX_ROOTS = 32;
  var timer = null;
  var queued = [];

  function schedule(roots) {
    for (var i = 0; i < roots.length && queued.length <= MAX_ROOTS; i++) {
      queued.push(roots[i]);
    }
    if (queued.length > MAX_ROOTS) queued = [document];
    if (timer !== null) return;
    timer = setTimeout(function () {
      timer = null;
      var roots = queued;
      queued = [];
      try {
        sweep(roots);
      } catch (e) {}
    }, SWEEP_MS);
  }

  function sweepAll() {
    schedule([document]);
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', sweepAll, { once: true });
  } else {
    sweepAll();
  }

  var observer = new MutationObserver(function (records) {
    var roots = [];
    for (var i = 0; i < records.length; i++) {
      var added = records[i].addedNodes;
      for (var j = 0; j < added.length; j++) {
        if (added[j].nodeType === 1) roots.push(added[j]);
      }
    }
    if (roots.length) schedule(roots);
  });

  function observe() {
    if (document.body) observer.observe(document.body, { childList: true, subtree: true });
  }
  if (document.body) observe();
  else document.addEventListener('DOMContentLoaded', observe, { once: true });

  setTimeout(function () {
    observer.disconnect();
    if (timer !== null) {
      clearTimeout(timer);
      timer = null;
    }
    queued = [];
  }, 20000);
})();
