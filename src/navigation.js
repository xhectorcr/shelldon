import { state } from './state.js';
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
window.navigate = navigate;