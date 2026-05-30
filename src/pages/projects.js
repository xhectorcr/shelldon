import { state } from '../state.js';
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
  div.innerHTML = `
    <div class="card-accent"></div>
    <div class="card-header">
      <div class="card-icon" style="color:${color};background:${color}11">⚡</div>
      <div class="card-actions">
        <button class="icon-btn" onclick="event.stopPropagation();editProject('${esc(p.name)}')" title="Editar">✏️</button>
        <button class="icon-btn danger" onclick="event.stopPropagation();deleteProject('${esc(p.name)}')" title="Eliminar">🗑️</button>
      </div>
    </div>
    <h3 class="card-title">${esc(p.name)}</h3>
    <p class="card-desc">${esc(p.description || 'Sin descripción')}</p>
    <div class="card-panes">${(p.panes||[]).map(pn=>`<span class="pane-chip">${esc(pn.name)}</span>`).join('')}</div>
    <div class="card-footer">
      <span class="terminal-badge">⌨ ${esc(p.terminal || 'default')}</span>
      <button class="launch-btn" style="background:${color}" onclick="event.stopPropagation();openLaunchModal(${JSON.stringify(p).replace(/"/g,'&quot;')})">
        🚀 Launch
      </button>
    </div>`;
  div.addEventListener('click', () => openLaunchModal(p));
  return div;
}

export async function deleteProject(name) {
  if (!confirm(`¿Eliminar "${name}"?`)) return;
  try { await invoke('delete_project', { name }); toast('🗑️', `"${name}" eliminado`); await loadProjects(); }
  catch(e) { toast('❌', 'Error: ' + e); }
}

window.filterProjects = filterProjects;
window.deleteProject = deleteProject;