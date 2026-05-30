let _tt;
export function toast(icon, msg) {
  clearTimeout(_tt);
  document.getElementById('toast-icon').textContent = icon;
  document.getElementById('toast-msg').textContent = msg;
  const t = document.getElementById('toast');
  t.classList.remove('hidden'); t.classList.add('show');
  _tt = setTimeout(() => { t.classList.remove('show'); setTimeout(() => t.classList.add('hidden'), 300); }, 3000);
}
window.toast = toast;