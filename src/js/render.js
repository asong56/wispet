export const ICONS = {
  database: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor"><path d="M128,26C75.29,26,34,49.72,34,80v96c0,30.28,41.29,54,94,54s94-23.72,94-54V80C222,49.72,180.71,26,128,26Zm0,12c44.45,0,82,19.23,82,42s-37.55,42-82,42S46,102.77,46,80,83.55,38,128,38Zm82,138c0,22.77-37.55,42-82,42s-82-19.23-82-42V154.79C62,171.16,92.37,182,128,182s66-10.84,82-27.21Zm0-48c0,22.77-37.55,42-82,42s-82-19.23-82-42V106.79C62,123.16,92.37,134,128,134s66-10.84,82-27.21Z"/></svg>`,
  globe: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor"><path d="M128,26A102,102,0,1,0,230,128,102.12,102.12,0,0,0,128,26Zm89.8,96H173.89c-1.54-40.77-18.48-68.23-30.43-82.67A90.19,90.19,0,0,1,217.8,122ZM128,215.83a110,110,0,0,1-15.19-19.45A128.37,128.37,0,0,1,94.13,134h67.74a128.37,128.37,0,0,1-18.68,62.38A110,110,0,0,1,128,215.83ZM94.13,122a128.37,128.37,0,0,1,18.68-62.38A110,110,0,0,1,128,40.17a110,110,0,0,1,15.19,19.45A128.37,128.37,0,0,1,161.87,122Zm18.41-82.67c-12,14.44-28.89,41.9-30.43,82.67H38.2A90.19,90.19,0,0,1,112.54,39.33ZM38.2,134H82.11c1.54,40.77,18.48,68.23,30.43,82.67A90.19,90.19,0,0,1,38.2,134Zm105.26,82.67c11.95-14.44,28.89-41.9,30.43-82.67H217.8A90.19,90.19,0,0,1,143.46,216.67Z"/></svg>`,
  article: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor"><path d="M216,42H40A14,14,0,0,0,26,56V200a14,14,0,0,0,14,14H216a14,14,0,0,0,14-14V56A14,14,0,0,0,216,42Zm2,158a2,2,0,0,1-2,2H40a2,2,0,0,1-2-2V56a2,2,0,0,1,2-2H216a2,2,0,0,1,2,2ZM180,96a6,6,0,0,1-6,6H82a6,6,0,0,1,0-12h92A6,6,0,0,1,180,96Zm0,32a6,6,0,0,1-6,6H82a6,6,0,0,1,0-12h92A6,6,0,0,1,180,128Zm0,32a6,6,0,0,1-6,6H82a6,6,0,0,1,0-12h92A6,6,0,0,1,180,160Z"/></svg>`,
  copy: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" fill="currentColor"><path d="M184,66H40a6,6,0,0,0-6,6V216a6,6,0,0,0,6,6H184a6,6,0,0,0,6-6V72A6,6,0,0,0,184,66Zm-6,144H46V78H178ZM222,40V184a6,6,0,0,1-12,0V46H72a6,6,0,0,1,0-12H216A6,6,0,0,1,222,40Z"/></svg>`,
};

const KIND_ICON = {
  dict: ICONS.database,
  translation: ICONS.globe,
  article: ICONS.article,
};

const ALLOWED_TAGS = new Set([
  'p','br','span','div','b','strong','i','em','u','s','sup','sub',
  'ul','ol','li','dl','dt','dd','table','thead','tbody','tr','th','td',
  'hr','blockquote','a','h1','h2','h3','h4','h5','h6',
  'img','abbr','cite','q','small','code','pre','ruby','rt','rp',
]);

const ALLOWED_ATTRS = new Set(['href','src','alt','title','lang','class','colspan','rowspan']);

export function sanitizeHtml(html) {
  const doc = new DOMParser().parseFromString(html, 'text/html');
  doc.querySelectorAll('script,style,link,meta,object,iframe,noscript').forEach(el => el.remove());

  const walker = doc.createTreeWalker(doc.body, NodeFilter.SHOW_ELEMENT);
  const toRemove = [];

  let node = walker.nextNode();
  while (node) {
    const tag = node.tagName.toLowerCase();
    if (!ALLOWED_TAGS.has(tag)) {
      toRemove.push(node);
    } else {
      for (const attr of [...node.attributes]) {
        if (!ALLOWED_ATTRS.has(attr.name.toLowerCase())) {
          node.removeAttribute(attr.name);
        }
        if (attr.name === 'href' && attr.value.startsWith('javascript:')) {
          node.removeAttribute('href');
        }
      }
    }
    node = walker.nextNode();
  }

  // Bottom-up so child disallowed nodes unwrap before their parent —
  // otherwise replaceWith can leave orphaned subtrees.
  for (let i = toRemove.length - 1; i >= 0; i--) {
    const el = toRemove[i];
    el.replaceWith(...el.childNodes);
  }

  return doc.body.innerHTML;
}

export function renderResult(result) {
  const section = document.createElement('section');
  section.className = 'result';
  section.dataset.providerId = result.provider_id;

  const header = document.createElement('div');
  header.className = 'provider-header';

  const icon = document.createElement('span');
  icon.innerHTML = KIND_ICON[result.kind] ?? ICONS.globe;
  header.appendChild(icon);

  const name = document.createElement('span');
  name.className = 'provider-name';
  name.textContent = result.provider_label;
  header.appendChild(name);

  if (result.phonetic) {
    const ph = document.createElement('span');
    ph.className = 'phonetic';
    ph.textContent = result.phonetic;
    header.appendChild(ph);
  }

  if (result.source_lang && result.kind === 'translation') {
    const lang = document.createElement('span');
    lang.className = 'badge';
    lang.textContent = result.source_lang.toUpperCase();
    lang.style.marginLeft = 'auto';
    header.appendChild(lang);
  }

  const copyBtn = document.createElement('button');
  copyBtn.className = 'copy-btn icon-btn';
  copyBtn.setAttribute('aria-label', 'Copy');
  copyBtn.innerHTML = ICONS.copy;
  copyBtn.addEventListener('click', () => copyContent(result.content, copyBtn));
  header.appendChild(copyBtn);

  section.appendChild(header);

  const content = document.createElement('div');
  if (result.kind === 'dict') {
    content.className = 'dict-content';
    content.innerHTML = sanitizeHtml(result.content);
    attachAudioLinks(content);
  } else if (result.kind === 'translation') {
    content.className = 'translation-content';
    content.textContent = result.content;
  } else {
    content.className = 'article-content';
    content.innerHTML = sanitizeHtml(result.content);
    attachAudioLinks(content);
  }
  section.appendChild(content);

  return section;
}

let sharedAudio = null;

// MDX pronunciation links are rewritten server-side to wispet://mdd/sound/...
// (see sanitize_mdx_html in provider/mdx.rs); intercept clicks and play
// through a shared <audio> instead of letting them navigate the window.
function attachAudioLinks(container) {
  container.addEventListener('click', (e) => {
    const link = e.target.closest('a[href^="wispet://mdd/sound/"]');
    if (!link) return;
    e.preventDefault();
    if (!sharedAudio) {
      sharedAudio = new Audio();
    }
    sharedAudio.src = link.getAttribute('href');
    sharedAudio.play().catch(() => {});
  });
}

export function renderLoading(label) {
  const section = document.createElement('section');
  section.className = 'result-loading';

  const d1 = document.createElement('div'); d1.className = 'loading-dot';
  const d2 = document.createElement('div'); d2.className = 'loading-dot';
  const d3 = document.createElement('div'); d3.className = 'loading-dot';
  section.appendChild(d1);
  section.appendChild(d2);
  section.appendChild(d3);

  const lbl = document.createElement('span');
  lbl.className = 'loading-label';
  lbl.textContent = label;
  section.appendChild(lbl);

  return section;
}

async function copyContent(content, btn) {
  const tmp = document.createElement('div');
  tmp.innerHTML = content;
  const text = tmp.textContent || tmp.innerText || content;

  try {
    await navigator.clipboard.writeText(text.trim());
    btn.style.color = 'var(--color-accent)';
    setTimeout(() => { btn.style.color = ''; }, 1200);
  } catch {
    const ta = document.createElement('textarea');
    ta.value = text.trim();
    ta.style.position = 'fixed';
    ta.style.opacity = '0';
    document.body.appendChild(ta);
    ta.select();
    document.execCommand('copy');
    document.body.removeChild(ta);
  }
}
