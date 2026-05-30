import { navigate } from './navigation.js';
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
});