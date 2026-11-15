/* Authenticated Atlantis room. Joining is explicit; no private coordinates are sent. */
(() => {
  const scene = () => {
    const value = window.ClubScene?.() || 'Atlantis:surface';
    return value === 'Atlantis:dungeon:1' ? value : 'Atlantis:surface';
  };
  let joined = false, activeScene = '', busy = false, lastMessage = 0;
  const toggle = document.createElement('button');
  toggle.type = 'button'; toggle.textContent = 'Club';
  toggle.setAttribute('aria-expanded', 'false');
  toggle.style.cssText = 'position:fixed;z-index:35;left:max(10px,env(safe-area-inset-left));bottom:calc(94px + env(safe-area-inset-bottom));min-width:48px;min-height:44px;padding:8px 12px;color:#e8f8ff;background:#071725dd;border:1px solid #3f8198;border-radius:12px;font:700 13px system-ui';
  const panel = document.createElement('section');
  panel.hidden = true; panel.setAttribute('aria-label', 'Atlantis club room');
  panel.style.cssText = 'position:fixed;z-index:35;left:max(10px,env(safe-area-inset-left));bottom:calc(145px + env(safe-area-inset-bottom));width:min(310px,calc(100vw - 20px));max-height:min(55vh,460px);overflow:auto;padding:12px;color:#e8f8ff;background:#071725f2;border:1px solid #3f8198;border-radius:12px;font:13px system-ui';
  const title = document.createElement('strong'); title.textContent = 'Atlantis club';
  const status = document.createElement('p'); status.textContent = 'Sign in to join.'; status.style.margin = '8px 0';
  const join = document.createElement('button'); join.type = 'button'; join.textContent = 'Join room';
  const roster = document.createElement('div'); roster.textContent = 'Presence unavailable'; roster.style.margin = '8px 0';
  const messages = document.createElement('div'); messages.style.cssText = 'max-height:180px;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere';
  const form = document.createElement('form'); form.style.cssText = 'display:flex;gap:6px;margin-top:8px';
  const input = document.createElement('input'); input.type = 'text'; input.maxLength = 500; input.placeholder = 'Message the room'; input.style.cssText = 'min-width:0;flex:1;padding:8px';
  const send = document.createElement('button'); send.type = 'submit'; send.textContent = 'Send';
  form.append(input, send); panel.append(title, status, join, roster, messages, form);
  document.body.append(toggle, panel);
  for (const element of [toggle, panel]) element.addEventListener('pointerdown', event => event.stopPropagation());
  toggle.addEventListener('click', () => {
    panel.hidden = !panel.hidden;
    toggle.setAttribute('aria-expanded', String(!panel.hidden));
    if (!panel.hidden) refresh();
  });
  async function leave() {
    if (joined) {
      joined = false; activeScene = ''; lastMessage = 0; messages.replaceChildren();
      try { await window.PlatformAuth.request('/api/auth/club/leave', {}); } catch {}
    }
    join.textContent = 'Join room'; roster.textContent = 'Presence unavailable';
  }
  join.addEventListener('click', async () => {
    if (!window.PlatformAuth?.profile) { window.PlatformAuth?.signIn(); return; }
    if (joined) { await leave(); status.textContent = 'Left the room.'; return; }
    try {
      activeScene = scene();
      await window.PlatformAuth.request('/api/auth/club/presence', {scene: activeScene});
      joined = true; join.textContent = 'Leave room'; status.textContent = 'Connected to ' + activeScene;
      await refresh();
    } catch (error) { status.textContent = error.message; }
  });
  form.addEventListener('submit', async event => {
    event.preventDefault(); event.stopPropagation();
    if (!joined || !input.value.trim()) return;
    send.disabled = true;
    try {
      await window.PlatformAuth.request('/api/auth/club/messages', {scene: activeScene, text: input.value});
      input.value = ''; await refresh();
    } catch (error) { status.textContent = error.message; }
    finally { send.disabled = false; }
  });
  async function refresh() {
    if (busy || !joined || !window.PlatformAuth?.profile) return;
    busy = true;
    try {
      const nextScene = scene();
      if (nextScene !== activeScene) {
        activeScene = nextScene; lastMessage = 0; messages.replaceChildren();
      }
      await window.PlatformAuth.request('/api/auth/club/presence', {scene: activeScene});
      const peers = await window.PlatformAuth.request('/api/auth/club/presence?scene=' + encodeURIComponent(activeScene));
      roster.textContent = peers.length + ' signed-in player' + (peers.length === 1 ? '' : 's') + ' in ' + activeScene;
      window.dispatchEvent(new CustomEvent('club-presence-update', {detail: {scene: activeScene, count: peers.length}}));
      if (!panel.hidden) {
        const feed = await window.PlatformAuth.request('/api/auth/club/messages?scene=' + encodeURIComponent(activeScene) + '&after=' + lastMessage);
        for (const message of feed) {
          const row = document.createElement('p');
          row.textContent = message.principal_did.slice(0, 24) + ': ' + message.text;
          messages.append(row); lastMessage = Math.max(lastMessage, message.id);
        }
        while (messages.childElementCount > 100) messages.firstChild.remove();
        messages.scrollTop = messages.scrollHeight;
      }
      status.textContent = 'Connected to ' + activeScene;
    } catch (error) { roster.textContent = 'Presence unavailable'; status.textContent = error.message; }
    finally { busy = false; }
  }
  window.addEventListener('platform-auth-change', event => {
    if (!event.detail?.profile) { joined = false; activeScene = ''; roster.textContent = 'Presence unavailable';
      status.textContent = 'Sign in to join.'; join.textContent = 'Join room'; messages.replaceChildren(); lastMessage = 0; }
  });
  setInterval(refresh, 12000);
})();
