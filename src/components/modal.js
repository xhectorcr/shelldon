import { state } from '../state.js';
import { toast } from './toast.js';
import { esc } from '../utils.js';

export function openLaunchModal(p) {
  state.pendingLaunch = p.name;
  document.getElementById('modal-icon').textContent = '⚡';
  document.getElementById('modal-title').textContent = p.name;
  document.getElementById('modal-desc').textContent = p.description || '';
  document.getElementById('modal-terminal').textContent = p.terminal || 'default';
  const el = document.getElementById('modal-panes');
  el.innerHTML = (p.panes || []).map((pn, i) => `
    <div class="modal-pane">
      <span class="pane-num">${i+1}</span>
      <div class="pane-info">
        <strong>${esc(pn.name)}</strong>
        <span class="pane-path">${esc(pn.path)}</span>
      </div>
      <code class="pane-cmd">${esc(pn.command)}</code>
    </div>`).join('');
  document.getElementById('launch-modal').classList.remove('hidden');
}

export function closeLaunchModal() {
  document.getElementById('launch-modal').classList.add('hidden');
  state.pendingLaunch = null;
}

export async function confirmLaunch() {
  if (!state.pendingLaunch) return;
  const btn = document.getElementById('btn-confirm');
  btn.disabled = true; btn.textContent = '⏳ Lanzando…';
  try {
    await invoke('launch_project', { name: state.pendingLaunch });
    closeLaunchModal();
    toast('🚀', `"${state.pendingLaunch}" lanzado!`);
  } catch(e) { toast('❌', 'Error: ' + e); }
  finally { btn.disabled = false; btn.textContent = '🚀 Lanzar'; }
}

window.openLaunchModal = openLaunchModal;
window.closeLaunchModal = closeLaunchModal;
window.confirmLaunch = confirmLaunch;