const API_BASE = 'http://localhost:8080';

const dropZone = document.getElementById('drop-zone');
const fileInput = document.getElementById('file-input');
const uploadList = document.getElementById('upload-list');
const searchInput = document.getElementById('search-input');
const searchBtn = document.getElementById('search-btn');
const searchResults = document.getElementById('search-results');
const documentsList = document.getElementById('documents-list');

dropZone.addEventListener('click', () => fileInput.click());

dropZone.addEventListener('dragover', (e) => {
    e.preventDefault();
    dropZone.classList.add('dragover');
});

dropZone.addEventListener('dragleave', () => {
    dropZone.classList.remove('dragover');
});

dropZone.addEventListener('drop', (e) => {
    e.preventDefault();
    dropZone.classList.remove('dragover');
    const files = e.dataTransfer.files;
    handleFiles(files);
});

fileInput.addEventListener('change', (e) => {
    handleFiles(e.target.files);
});

async function handleFiles(files) {
    for (const file of files) {
        await uploadFile(file);
    }
    loadDocuments();
}

async function uploadFile(file) {
    const formData = new FormData();
    formData.append('file', file);

    try {
        const response = await fetch(`${API_BASE}/api/documents`, {
            method: 'POST',
            body: formData
        });

        const result = await response.json();
        
        const item = document.createElement('div');
        item.className = 'upload-item';
        item.innerHTML = `
            <span>${file.name}</span>
            <span class="${response.ok ? 'success' : 'error'}">
                ${response.ok ? 'Uploaded' : 'Failed'}
            </span>
        `;
        uploadList.appendChild(item);
    } catch (error) {
        console.error('Upload error:', error);
        const item = document.createElement('div');
        item.className = 'upload-item';
        item.innerHTML = `
            <span>${file.name}</span>
            <span class="error">Error: ${error.message}</span>
        `;
        uploadList.appendChild(item);
    }
}

searchBtn.addEventListener('click', async () => {
    const query = searchInput.value.trim();
    if (!query) return;

    try {
        const response = await fetch(`${API_BASE}/api/search`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ query, limit: 10 })
        });

        const result = await response.json();
        
        searchResults.innerHTML = '';
        
        if (result.results && result.results.length > 0) {
            result.results.forEach(r => {
                const item = document.createElement('div');
                item.className = 'result-item';
                item.innerHTML = `
                    <h3>${r.document_name || 'Document'}</h3>
                    <p>${r.chunk_text.substring(0, 200)}...</p>
                    <small>Score: ${r.score.toFixed(3)}</small>
                `;
                searchResults.appendChild(item);
            });
        } else {
            searchResults.innerHTML = '<p>No results found</p>';
        }
    } catch (error) {
        console.error('Search error:', error);
        searchResults.innerHTML = `<p class="error">Error: ${error.message}</p>`;
    }
});

async function loadDocuments() {
    try {
        const response = await fetch(`${API_BASE}/api/documents`);
        const docs = await response.json();
        
        documentsList.innerHTML = '';
        
        if (docs && docs.length > 0) {
            docs.forEach(doc => {
                const item = document.createElement('div');
                item.className = 'doc-item';
                item.innerHTML = `
                    <span class="name">${doc.name}</span>
                    <span class="meta">${doc.source} | ${new Date(doc.created_at).toLocaleDateString()}</span>
                `;
                documentsList.appendChild(item);
            });
        } else {
            documentsList.innerHTML = '<p>No documents uploaded yet</p>';
        }
    } catch (error) {
        console.error('Load documents error:', error);
    }
}

loadDocuments();
