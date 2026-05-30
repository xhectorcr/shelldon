export function setLayout(val) {
  localStorage.setItem('shelldon:layout', val);
  document.querySelectorAll('#page-settings .layout-option').forEach(el => el.classList.remove('selected'));
  const el = document.getElementById('layout-' + val);
  if (el) el.classList.add('selected');
}
window.setLayout = setLayout;