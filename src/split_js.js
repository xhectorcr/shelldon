const fs = require('fs');
const path = require('path');

const srcDir = __dirname;

const stateJs = `export const state = { projects: [], editing: null, panes: [], pendingLaunch: null, formLayout: 'cols-2' };`;

const utilsJs = `export function esc(s) {
  return String(s || '').replace(/&/g,'&amp;').replace(/"/g,'&quot;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
}`;

const toastJs = `let _tt;
export function toast(icon, msg) {
  clearTimeout(_tt);
  document.getElementById('toast-icon').textContent = icon;
  document.getElementById('toast-msg').textContent = msg;
  const t = document.getElementById('toast');
  t.classList.remove('hidden'); t.classList.add('show');
  _tt = setTimeout(() => { t.classList.remove('show'); setTimeout(() => t.classList.add('hidden'), 300); }, 3000);
}
window.toast = toast;`;

const modalJs = `import { state } from '../state.js';
import { toast } from './toast.js';
import { esc } from '../utils.js';

export function openLaunchModal(p) {
  state.pendingLaunch = p.name;
  document.getElementById('modal-icon').textContent = '⚡';
  document.getElementById('modal-title').textContent = p.name;
  document.getElementById('modal-desc').textContent = p.description || '';
  document.getElementById('modal-terminal').textContent = p.terminal || 'default';
  const el = document.getElementById('modal-panes');
  el.innerHTML = (p.panes || []).map((pn, i) => \`
    <div class="modal-pane">
      <span class="pane-num">\${i+1}</span>
      <div class="pane-info">
        <strong>\${esc(pn.name)}</strong>
        <span class="pane-path">\${esc(pn.path)}</span>
      </div>
      <code class="pane-cmd">\${esc(pn.command)}</code>
    </div>\`).join('');
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
    toast('🚀', \`"\${state.pendingLaunch}" lanzado!\`);
  } catch(e) { toast('❌', 'Error: ' + e); }
  finally { btn.disabled = false; btn.textContent = '🚀 Lanzar'; }
}

window.openLaunchModal = openLaunchModal;
window.closeLaunchModal = closeLaunchModal;
window.confirmLaunch = confirmLaunch;`;

const projectsJs = `import { state } from '../state.js';
import { toast } from '../components/toast.js';
import { esc } from '../utils.js';
import { editProject } from './new.js';
import { openLaunchModal } from '../components/modal.js';

export async function loadProjects() {
  try {
    state.projects = await invoke('get_projects');
    renderProjects();
  } catch(e) { toast('❌', 'Error cargando proyectos: ' + e); }
}

export function filterProjects() {
  renderProjects();
}

export function renderProjects() {
  const q = (document.getElementById('search-input').value || '').toLowerCase();
  const list = state.projects.filter(p =>
    p.name.toLowerCase().includes(q) || (p.description || '').toLowerCase().includes(q)
  );
  const grid = document.getElementById('projects-grid');
  const empty = document.getElementById('empty-state');
  grid.innerHTML = '';
  if (list.length === 0) { empty.style.display = 'flex'; return; }
  empty.style.display = 'none';
  list.forEach(p => grid.appendChild(buildCard(p)));
}

function buildCard(p) {
  const color = p.color || '#7c3aed';
  const div = document.createElement('div');
  div.className = 'project-card';
  div.style.setProperty('--card-color', color);
  div.innerHTML = \`
    <div class="card-accent"></div>
    <div class="card-header">
      <div class="card-icon" style="color:\${color};background:\${color}11">⚡</div>
      <div class="card-actions">
        <button class="icon-btn" onclick="event.stopPropagation();editProject('\${esc(p.name)}')" title="Editar">✏️</button>
        <button class="icon-btn danger" onclick="event.stopPropagation();deleteProject('\${esc(p.name)}')" title="Eliminar">🗑️</button>
      </div>
    </div>
    <h3 class="card-title">\${esc(p.name)}</h3>
    <p class="card-desc">\${esc(p.description || 'Sin descripción')}</p>
    <div class="card-panes">\${(p.panes||[]).map(pn=>\`<span class="pane-chip">\${esc(pn.name)}</span>\`).join('')}</div>
    <div class="card-footer">
      <span class="terminal-badge">⌨ \${esc(p.terminal || 'default')}</span>
      <button class="launch-btn" style="background:\${color}" onclick="event.stopPropagation();openLaunchModal(\${JSON.stringify(p).replace(/"/g,'&quot;')})">
        🚀 Launch
      </button>
    </div>\`;
  div.addEventListener('click', () => openLaunchModal(p));
  return div;
}

export async function deleteProject(name) {
  if (!confirm(\`¿Eliminar "\${name}"?\`)) return;
  try { await invoke('delete_project', { name }); toast('🗑️', \`"\${name}" eliminado\`); await loadProjects(); }
  catch(e) { toast('❌', 'Error: ' + e); }
}

window.filterProjects = filterProjects;
window.deleteProject = deleteProject;`;

const newJs = `import { state } from '../state.js';
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
    const cb = document.querySelector(\`.extra-cb[value="\${ex}"]\`);
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
    if (!pn.name.trim() || !pn.path.trim() || !pn.command.trim()) { toast('⚠️', \`Pane \${i+1} incompleto\`); return; }
  }
  const extras = [];
  document.querySelectorAll('.extra-cb:checked').forEach(cb => extras.push(cb.value));
  const burl = document.getElementById('field-browser-url').value.trim();
  if (burl) extras.push('browser:' + burl);
  const project = {
    name,
    description: document.getElementById('field-description').value.trim(),
    icon: '⚡',
    color: document.getElementById('field-color').value,
    terminal: document.getElementById('field-terminal').value,
    layout: state.formLayout,
    panes: state.panes.map(p => ({...p})),
    extras
  };
  try {
    await invoke('save_project', { project });
    toast('✅', \`Proyecto "\${name}" guardado\`);
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
  list.innerHTML = state.panes.map((pn, i) => \`
    <div class="pane-row">
      <div class="pane-row-header">
        <span class="pane-label">Pane \${i+1}</span>
        <button type="button" class="remove-pane" onclick="removePane(\${i})">✕ Quitar</button>
      </div>
      <div class="pane-fields">
        <div class="field-group">
          <label>Nombre</label>
          <input class="pane-name-inp form-input" placeholder="Frontend" value="\${esc(pn.name)}" oninput="updatePane(\${i},'name',this.value)">
        </div>
        <div class="field-group">
          <label>Directorio</label>
          <div style="display:flex;gap:6px">
            <input class="form-input mono" style="flex:1" placeholder="./frontend" value="\${esc(pn.path)}" oninput="updatePane(\${i},'path',this.value)">
            <button type="button" class="browse-btn" onclick="pickDir(\${i})" title="Explorar">📁</button>
          </div>
        </div>
        <div class="field-group" style="grid-column:1/-1">
          <label>Comando</label>
          <input class="form-input mono" placeholder="npm run dev" value="\${esc(pn.command)}" oninput="updatePane(\${i},'command',this.value)">
        </div>
      </div>
    </div>\`).join('');
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
window.updatePane = updatePane;`;

const settingsJs = `export function setLayout(val) {
  localStorage.setItem('shelldon:layout', val);
  document.querySelectorAll('#page-settings .layout-option').forEach(el => el.classList.remove('selected'));
  const el = document.getElementById('layout-' + val);
  if (el) el.classList.add('selected');
}
window.setLayout = setLayout;`;

const navigationJs = `import { state } from './state.js';
import { loadProjects } from './pages/projects.js';
import { resetForm } from './pages/new.js';
import { setLayout } from './pages/settings.js';

export function navigate(page) {
  document.querySelectorAll('.page').forEach(el => el.classList.add('hidden'));
  const target = document.getElementById('page-' + page);
  if(target) target.classList.remove('hidden');
  
  document.querySelectorAll('.nav-btn').forEach(b => b.classList.remove('active-nav'));
  const nb = document.getElementById('nav-' + page);
  if (nb) nb.classList.add('active-nav');
  
  if (page === 'new' && !state.editing) resetForm();
  if (page === 'projects') { state.editing = null; loadProjects(); }
  if (page === 'settings') {
    const term = localStorage.getItem('shelldon:terminal') || 'windows-terminal';
    const stEl = document.getElementById('setting-terminal');
    if (stEl) stEl.value = term;
    
    const layout = localStorage.getItem('shelldon:layout') || 'horizontal';
    setLayout(layout);
  }
}
window.navigate = navigate;`;

const mainJs = `import { navigate } from './navigation.js';
import { loadProjects } from './pages/projects.js';
import { closeLaunchModal } from './components/modal.js';
import './pages/new.js';
import './pages/settings.js';

document.addEventListener('partialsLoaded', async () => {
  const colorField = document.getElementById('field-color');
  if(colorField) {
    colorField.addEventListener('input', e => {
      document.getElementById('color-hex').textContent = e.target.value;
    });
  }
  
  document.addEventListener('keydown', e => {
    if (e.key === 'Escape') closeLaunchModal();
    if ((e.ctrlKey || e.metaKey) && e.key === 'n') { e.preventDefault(); navigate('new'); }
  });
  
  await loadProjects();
  navigate('projects');
});`;

fs.writeFileSync(path.join(srcDir, 'state.js'), stateJs);
fs.writeFileSync(path.join(srcDir, 'utils.js'), utilsJs);
fs.writeFileSync(path.join(srcDir, 'components', 'toast.js'), toastJs);
fs.writeFileSync(path.join(srcDir, 'components', 'modal.js'), modalJs);
fs.writeFileSync(path.join(srcDir, 'pages', 'projects.js'), projectsJs);
fs.writeFileSync(path.join(srcDir, 'pages', 'new.js'), newJs);
fs.writeFileSync(path.join(srcDir, 'pages', 'settings.js'), settingsJs);
fs.writeFileSync(path.join(srcDir, 'navigation.js'), navigationJs);
fs.writeFileSync(path.join(srcDir, 'main.js'), mainJs);

let indexHtml = fs.readFileSync(path.join(srcDir, 'index.html'), 'utf8');
indexHtml = indexHtml.replace('<script src="main.js"></script>', '<script type="module" src="main.js"></script>');
fs.writeFileSync(path.join(srcDir, 'index.html'), indexHtml);
console.log('JS refactoring done.');
