import { state } from '../state.js';
import { toast } from '../components/toast.js';
import { esc } from '../utils.js';
import { navigate } from '../navigation.js';

export function resetForm() {
  document.getElementById('form-title').textContent = 'Nuevo Proyecto';
  ['name','description','browser-url'].forEach(f => { const el = document.getElementById('field-'+f); if(el) el.value = ''; });
  document.getElementById('field-color').value = '#7c3aed';
  document.getElementById('color-hex').textContent = '#7c3aed';
  document.getElementById('field-terminal').value = localStorage.getItem('shelldon:terminal') || 'windows-terminal';
  setFormLayout(localStorage.getItem('shelldon:layout') || 'cols-2');
  document.querySelectorAll('.extra-cb').forEach(cb => cb.checked = false);
  state.panes = [];
  renderPanes();
}

export async function editProject(name) {
  state.editing = name;
  navigate('new');
  const p = await invoke('get_project_by_name', { name });
  if (!p) { toast('❌', 'Proyecto no encontrado'); return; }
  document.getElementById('form-title').textContent = 'Editar: ' + p.name;
  document.getElementById('field-name').value = p.name;
  document.getElementById('field-description').value = p.description || '';
  document.getElementById('field-color').value = p.color || '#7c3aed';
  document.getElementById('color-hex').textContent = p.color || '#7c3aed';
  document.getElementById('field-terminal').value = p.terminal || 'windows-terminal';
  setFormLayout(p.layout || 'cols-2');
  state.panes = (p.panes || []).map(x => ({...x}));
  renderPanes();
  document.querySelectorAll('.extra-cb').forEach(cb => { cb.checked = false; });
  (p.extras || []).forEach(ex => {
    const cb = document.querySelector(`.extra-cb[value="${ex}"]`);
    if (cb) cb.checked = true;
    if (ex.startsWith('browser:')) document.getElementById('field-browser-url').value = ex.replace('browser:', '');
  });
}

export async function saveProject(e) {
  e.preventDefault();
  const name = document.getElementById('field-name').value.trim();
  if (!name) { toast('⚠️', 'El nombre es requerido'); return; }
  if (state.panes.length === 0) { toast('⚠️', 'Agrega al menos un pane'); return; }
  for (const [i, pn] of state.panes.entries()) {
    if (!pn.name.trim() || !pn.path.trim() || !pn.command.trim()) { toast('⚠️', `Pane ${i+1} incompleto`); return; }
  }
  const extras = [];
  document.querySelectorAll('.extra-cb:checked').forEach(cb => extras.push(cb.value));
  const burl = document.getElementById('field-browser-url').value.trim();
  if (burl) extras.push('browser:' + burl);
  let finalLayout = state.formLayout;
  if (state.panes.length > 2) {
    finalLayout = 'grid-2x2';
  }

  const project = {
    name,
    description: document.getElementById('field-description').value.trim(),
    icon: '⚡',
    color: document.getElementById('field-color').value,
    terminal: document.getElementById('field-terminal').value,
    layout: finalLayout,
    panes: state.panes.map(p => ({...p})),
    extras
  };
  try {
    await invoke('save_project', { project });
    toast('✅', `Proyecto "${name}" guardado`);
    state.editing = null;
    navigate('projects');
  } catch(e) { toast('❌', 'Error: ' + e); }
}

export function addPane() {
  state.panes.push({ name: '', path: '', command: '' });
  renderPanes();
  const inputs = document.querySelectorAll('.pane-name-inp');
  if (inputs.length) inputs[inputs.length-1].focus();
}

export function removePane(i) { state.panes.splice(i, 1); renderPanes(); }

export function renderPanes() {
  const list = document.getElementById('panes-list');
  document.getElementById('panes-hint').style.display = state.panes.length ? 'none' : 'block';
  list.innerHTML = state.panes.map((pn, i) => `
    <div class="pane-row">
      <div class="pane-row-header">
        <span class="pane-label">Pane ${i+1}</span>
        <button type="button" class="remove-pane" onclick="removePane(${i})">✕ Quitar</button>
      </div>
      <div class="pane-fields">
        <div class="field-group">
          <label>Nombre</label>
          <input class="pane-name-inp form-input" placeholder="Frontend" value="${esc(pn.name)}" oninput="updatePane(${i},'name',this.value)">
        </div>
        <div class="field-group">
          <label>Directorio</label>
          <div style="display:flex;gap:6px">
            <input class="form-input mono" style="flex:1" placeholder="./frontend" value="${esc(pn.path)}" oninput="updatePane(${i},'path',this.value)">
            <button type="button" class="browse-btn" onclick="pickDir(${i})" title="Explorar">📁</button>
          </div>
        </div>
        <div class="field-group" style="grid-column:1/-1">
          <label>Comando</label>
          <input class="form-input mono" placeholder="npm run dev" value="${esc(pn.command)}" oninput="updatePane(${i},'command',this.value)">
        </div>
      </div>
    </div>`).join('');
}

export async function pickDir(i) {
  try {
    const path = await invoke('open_directory_dialog');
    if (path) { state.panes[i].path = path; renderPanes(); }
  } catch(_) {}
}

export function setFormLayout(val) {
  state.formLayout = val;
  document.querySelectorAll('#page-new .layout-option').forEach(el => el.classList.remove('selected'));
  const el = document.getElementById('form-layout-' + val);
  if (el) el.classList.add('selected');
}

export function updatePane(i, field, val) {
  state.panes[i][field] = val;
}

window.editProject = editProject;
window.saveProject = saveProject;
window.addPane = addPane;
window.removePane = removePane;
window.pickDir = pickDir;
window.setFormLayout = setFormLayout;
window.updatePane = updatePane;