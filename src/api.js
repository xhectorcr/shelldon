const isTauri = typeof window.__TAURI__ !== 'undefined';

async function invoke(cmd, args = {}) {
  if (isTauri) return window.__TAURI__.invoke(cmd, args);
  return mockInvoke(cmd, args);
}

const _db = {
  projects: [
    { name: 'ERP', description: 'Laravel + Vue full-stack', terminal: 'auto', color: '#7c3aed', icon: '🏢',
      panes: [{ name: 'Frontend', path: 'C:/Projects/erp/frontend', command: 'npm run dev' }, { name: 'Backend', path: 'C:/Projects/erp/backend', command: 'php artisan serve' }], extras: [] },
    { name: 'Portfolio', description: 'Nuxt 3 — sitio personal', terminal: 'auto', color: '#0891b2', icon: '🌐',
      panes: [{ name: 'Dev Server', path: 'C:/Projects/portfolio', command: 'npm run dev' }], extras: [] },
    { name: 'Vision-AI', description: 'YOLOv8 + FastAPI + Flutter', terminal: 'auto', color: '#059669', icon: '👁️',
      panes: [{ name: 'API', path: 'C:/Projects/vision/api', command: 'uvicorn main:app --reload' }, { name: 'App', path: 'C:/Projects/vision/app', command: 'flutter run' }], extras: [] },
    { name: 'Listo-App', description: 'E-commerce con detección YOLO', terminal: 'auto', color: '#dc2626', icon: '🛒',
      panes: [{ name: 'Backend', path: 'C:/Projects/listo/api', command: 'python manage.py runserver' }, { name: 'Frontend', path: 'C:/Projects/listo/web', command: 'npm run dev' }, { name: 'YOLO', path: 'C:/Projects/listo/vision', command: 'python yolo_server.py' }], extras: [] },
  ]
};

async function mockInvoke(cmd, args = {}) {
  await new Promise(r => setTimeout(r, 80));
  switch (cmd) {
    case 'get_projects': return JSON.parse(JSON.stringify(_db.projects));
    case 'get_project_by_name': return JSON.parse(JSON.stringify(_db.projects.find(p => p.name.toLowerCase() === args.name.toLowerCase()) || null));
    case 'save_project': {
      const i = _db.projects.findIndex(p => p.name.toLowerCase() === args.project.name.toLowerCase());
      if (i >= 0) _db.projects[i] = { ..._db.projects[i], ...args.project };
      else _db.projects.push({ ...args.project });
      return null;
    }
    case 'delete_project': {
      const before = _db.projects.length;
      _db.projects = _db.projects.filter(p => p.name.toLowerCase() !== args.name.toLowerCase());
      return _db.projects.length < before;
    }
    case 'launch_project': await new Promise(r => setTimeout(r, 600)); return null;
    case 'open_directory_dialog': return 'C:/Projects/mi-proyecto';
    default: throw new Error(`Comando desconocido: ${cmd}`);
  }
}

const appWindow = isTauri
  ? window.__TAURI__.window.appWindow
  : { minimize: () => {}, toggleMaximize: () => {}, close: () => {} };

window.appWindow = appWindow;
