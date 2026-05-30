const fs = require('fs');
const path = require('path');

const srcDir = __dirname;
const indexHtmlPath = path.join(srcDir, 'index.html');
const mainJsPath = path.join(srcDir, 'main.js');

let indexHtml = fs.readFileSync(indexHtmlPath, 'utf8');

// 1. Extract CSS
const styleMatch = indexHtml.match(/<style>([\s\S]*?)<\/style>/);
if (styleMatch) {
    const cssContent = styleMatch[1].trim();
    const stylesDir = path.join(srcDir, 'styles');
    if (!fs.existsSync(stylesDir)) fs.mkdirSync(stylesDir, { recursive: true });
    fs.writeFileSync(path.join(stylesDir, 'main.css'), cssContent);
    
    // Replace <style>...</style> with <link rel="stylesheet" href="styles/main.css">
    indexHtml = indexHtml.replace(/<style>[\s\S]*?<\/style>/, '<link rel="stylesheet" href="styles/main.css" />');
}

// 2. Extract HTML pages
const pages = [
    { id: 'page-projects', file: 'projects.html' },
    { id: 'page-new', file: 'new.html' },
    { id: 'page-settings', file: 'settings.html' }
];

const pagesDir = path.join(srcDir, 'pages');
if (!fs.existsSync(pagesDir)) fs.mkdirSync(pagesDir, { recursive: true });

pages.forEach(page => {
    // Regex to match <div id="page.id" class="page">...</div> (assuming it ends before the next <!-- PAGE or </main>)
    // Since parsing HTML with regex is brittle, let's use a simpler substring approach
    const startTag = `<div id="${page.id}" class="page">`;
    const startIdx = indexHtml.indexOf(startTag);
    if (startIdx !== -1) {
        // find matching closing div
        let endIdx = -1;
        let depth = 0;
        let i = startIdx;
        while (i < indexHtml.length) {
            if (indexHtml.substring(i, i + 4) === '<div') {
                depth++;
                i += 4;
            } else if (indexHtml.substring(i, i + 5) === '</div') {
                depth--;
                if (depth === 0) {
                    endIdx = i + 6; // include </div>
                    break;
                }
                i += 5;
            } else {
                i++;
            }
        }
        
        if (endIdx !== -1) {
            let pageContent = indexHtml.substring(startIdx, endIdx);
            
            // Remove the wrapping div, or keep it?
            // The plan says we'll load them into the container. But maybe it's easier to keep the wrapping div in index.html,
            // and the content inside the file. Or keep the whole div in the file and index.html just has a placeholder.
            // Let's keep the whole div in the file, and put a placeholder in index.html.
            
            fs.writeFileSync(path.join(pagesDir, page.file), pageContent);
            indexHtml = indexHtml.substring(0, startIdx) + `<div class="page-container" data-src="pages/${page.file}"></div>` + indexHtml.substring(endIdx);
        }
    }
});

// Extract Modales
const componentsDir = path.join(srcDir, 'components');
if (!fs.existsSync(componentsDir)) fs.mkdirSync(componentsDir, { recursive: true });

const extractComponent = (startTag, file) => {
    const startIdx = indexHtml.indexOf(startTag);
    if (startIdx !== -1) {
        let endIdx = -1;
        let depth = 0;
        let i = startIdx;
        while (i < indexHtml.length) {
            if (indexHtml.substring(i, i + 4) === '<div') {
                depth++;
                i += 4;
            } else if (indexHtml.substring(i, i + 5) === '</div') {
                depth--;
                if (depth === 0) {
                    endIdx = i + 6;
                    break;
                }
                i += 5;
            } else {
                i++;
            }
        }
        
        if (endIdx !== -1) {
            let content = indexHtml.substring(startIdx, endIdx);
            fs.writeFileSync(path.join(componentsDir, file), content);
            indexHtml = indexHtml.substring(0, startIdx) + `<div class="component-container" data-src="components/${file}"></div>` + indexHtml.substring(endIdx);
        }
    }
}

extractComponent('<div id="launch-modal" class="hidden">', 'modal.html');
extractComponent('<div id="toast">', 'toast.html');

// Append script to load partials at the end of body, before <script src="api.js">
const loaderScript = `
  <script>
    async function loadPartials() {
      const els = document.querySelectorAll('[data-src]');
      for (const el of els) {
        const res = await fetch(el.getAttribute('data-src'));
        el.outerHTML = await res.text();
      }
    }
    loadPartials().then(() => {
        // Dispatch an event so main.js knows when DOM is fully loaded with partials
        document.dispatchEvent(new Event('partialsLoaded'));
    });
  </script>
`;

indexHtml = indexHtml.replace('<script src="api.js"></script>', loaderScript + '\  <script src="api.js"></script>');

fs.writeFileSync(indexHtmlPath, indexHtml);
console.log('HTML and CSS extraction done.');
