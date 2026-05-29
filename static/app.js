// Minimal Vanilla JS Frontend

let currentBookId = null;
let currentBookData = null;
let bionicEnabled = false;
let chapterBoundaries = [];
let peripheralChars = 20;
let peripheralBrightness = 0.5;

document.addEventListener('DOMContentLoaded', async () => {
    // 1. Fetch book metadata and populate TOC
    const urlParams = new URLSearchParams(window.location.search);
    currentBookId = urlParams.get('book');
    
    if (!currentBookId) {
        document.getElementById('library-view').classList.remove('hidden');
        document.getElementById('reader-container').classList.add('hidden');
        document.getElementById('sidebar').classList.add('hidden');
        
        // Load library
        try {
            const res = await fetch('/api/library');
            if (res.ok) {
                const books = await res.json();
                const list = document.getElementById('library-list');
                list.innerHTML = '';
                books.forEach(b => {
                    const progress = localStorage.getItem('progress_' + b.id) ? 'Started' : 'New';
                    list.innerHTML += `
                        <div class="book-item">
                            <div>
                                <h3>${b.title}</h3>
                                <p>${b.author || 'Unknown Author'} - ${progress}</p>
                            </div>
                            <button class="resume-btn" onclick="window.location.href='/?book=${b.id}'">Open</button>
                        </div>
                    `;
                });
            }
        } catch (e) { console.error("Failed to load library", e); }
        
        // Handle upload
        document.getElementById('book-upload').addEventListener('change', async (e) => {
            const file = e.target.files[0];
            if (!file) return;
            const buffer = await file.arrayBuffer();
            try {
                const res = await fetch('/api/library/upload', { method: 'POST', body: buffer });
                if (res.ok) {
                    const data = await res.json();
                    window.location.href = `/?book=${data.id}`;
                }
            } catch (err) { console.error("Upload failed", err); }
        });
        return;
    }
    
    try {
        const res = await fetch(`/api/book/${currentBookId}`);
        if (!res.ok) throw new Error('Book not found');
        currentBookData = await res.json();
        const book = currentBookData;
        
        document.getElementById('book-title').innerText = book.title;
        
        const tocContainer = document.getElementById('toc');
        book.chapters.forEach(chapter => {
            const link = document.createElement('a');
            link.href = "#";
            link.innerText = chapter.title || `Chapter ${chapter.order + 1}`;
            link.style.display = 'block';
            link.style.padding = '8px 0';
            link.style.textDecoration = 'none';
            link.style.color = 'inherit';
            link.addEventListener('click', (e) => {
                e.preventDefault();
                const target = document.getElementById(`chapter-${chapter.id}`);
                if (target) {
                    document.getElementById('chapter-content').scrollTop = target.offsetTop;
                }
            });
            tocContainer.appendChild(link);
        });
        
        // 2. Setup event listeners
        document.getElementById('bionic-toggle').addEventListener('change', (e) => {
            bionicEnabled = e.target.checked;
            if (currentBookData) loadAllChapters(currentBookData);
        });
        
        document.getElementById('theme-toggle').addEventListener('change', (e) => {
            if (e.target.checked) document.body.classList.add('theme-dark');
            else document.body.classList.remove('theme-dark');
        });

        document.getElementById('font-size').addEventListener('input', (e) => {
            document.documentElement.style.setProperty('--user-font-size', `${e.target.value}rem`);
        });

        document.getElementById('font-weight').addEventListener('input', (e) => {
            document.documentElement.style.setProperty('--user-font-weight', e.target.value);
        });

        document.getElementById('peripheral-chars').addEventListener('input', (e) => {
            peripheralChars = parseInt(e.target.value, 10);
            if (!speedReaderPlaying) updateSpeedReaderWord();
        });

        document.getElementById('peripheral-brightness').addEventListener('input', (e) => {
            peripheralBrightness = parseFloat(e.target.value);
            document.documentElement.style.setProperty('--peripheral-opacity', peripheralBrightness);
        });
        
        document.getElementById('toggle-sidebar').addEventListener('click', () => {
            document.getElementById('sidebar').classList.toggle('hidden');
        });

        document.getElementById('toggle-settings').addEventListener('click', () => {
            document.getElementById('settings-panel').classList.toggle('hidden');
        });

        // Speed Reader logic
        document.getElementById('start-speed-read').addEventListener('click', () => {
            document.getElementById('speed-reader-overlay').classList.remove('hidden');
            speedReaderWords = getSpeedReaderWords();
            
            const savedProgress = localStorage.getItem('progress_' + currentBookId);
            if (savedProgress !== null) {
                speedReaderIndex = parseInt(savedProgress, 10);
            } else if (speedReaderWords.length > 0 && speedReaderIndex >= speedReaderWords.length) {
                speedReaderIndex = 0;
            }
            
            speedReaderWpm = parseInt(document.getElementById('sr-wpm').value, 10);
            updateSpeedReaderWord();
        });

        document.getElementById('sr-close').addEventListener('click', () => {
            document.getElementById('speed-reader-overlay').classList.add('hidden');
            pauseSpeedReader();
        });

        document.getElementById('sr-rewind').addEventListener('click', () => {
            speedReaderIndex = Math.max(0, speedReaderIndex - 10);
            updateSpeedReaderWord();
        });

        document.getElementById('sr-forward').addEventListener('click', () => {
            speedReaderIndex = Math.min(speedReaderWords.length - 1, speedReaderIndex + 10);
            updateSpeedReaderWord();
        });

        document.getElementById('sr-prev-chapter').addEventListener('click', () => {
            let prev = 0;
            for (let i = chapterBoundaries.length - 1; i >= 0; i--) {
                if (chapterBoundaries[i] < speedReaderIndex) {
                    prev = chapterBoundaries[i];
                    break;
                }
            }
            speedReaderIndex = prev;
            updateSpeedReaderWord();
        });

        document.getElementById('sr-next-chapter').addEventListener('click', () => {
            let next = speedReaderWords.length - 1;
            for (let i = 0; i < chapterBoundaries.length; i++) {
                if (chapterBoundaries[i] > speedReaderIndex) {
                    next = chapterBoundaries[i];
                    break;
                }
            }
            speedReaderIndex = next;
            updateSpeedReaderWord();
        });

        document.getElementById('sr-progress-slider').addEventListener('input', (e) => {
            if (speedReaderWords.length > 0) {
                speedReaderIndex = Math.floor((parseInt(e.target.value, 10) / 100) * (speedReaderWords.length - 1));
                updateSpeedReaderWord();
            }
        });

        document.getElementById('sr-play-pause').addEventListener('click', () => {
            if (speedReaderPlaying) pauseSpeedReader();
            else playSpeedReader();
        });

        document.getElementById('sr-wpm').addEventListener('input', (e) => {
            speedReaderWpm = parseInt(e.target.value, 10);
            document.getElementById('sr-wpm-label').innerText = `${speedReaderWpm} WPM`;
            if (speedReaderPlaying) {
                pauseSpeedReader();
                playSpeedReader();
            }
        });

        // 3. Load all chapters
        if (currentBookData.chapters.length > 0) {
            loadAllChapters(currentBookData);
        }
    } catch (e) {
        console.error(e);
        document.getElementById('chapter-content').innerHTML = '<p>Error loading book.</p>';
    }
});

let speedReaderWords = [];
let speedReaderIndex = 0;
let speedReaderInterval = null;
let speedReaderWpm = 300;
let speedReaderPlaying = false;

function getSpeedReaderWords() {
    const container = document.getElementById('chapter-content');
    const chapters = container.querySelectorAll('.chapter-container');
    
    let words = [];
    chapterBoundaries = [];
    
    chapters.forEach(chap => {
        chapterBoundaries.push(words.length);
        if (bionicEnabled) {
            const nodes = chap.querySelectorAll('.br-word');
            nodes.forEach(node => {
                let html = node.outerHTML;
                let next = node.nextSibling;
                if (next && next.nodeType === Node.TEXT_NODE) {
                    let text = next.textContent.trim();
                    if (text && !text.match(/^\s/)) {
                        let punc = text.match(/^[^\w\s]+/);
                        if (punc) html += punc[0];
                    }
                }
                const tmp = document.createElement('DIV');
                tmp.innerHTML = html;
                words.push({ html, len: tmp.textContent.length });
            });
            if (nodes.length === 0) {
                let chapWords = chap.innerText.trim().split(/\s+/).filter(w => w.length > 0);
                chapWords.forEach(w => words.push({ html: w, len: w.length }));
            }
        } else {
            let chapWords = chap.innerText.trim().split(/\s+/).filter(w => w.length > 0);
            chapWords.forEach(w => words.push({ html: w, len: w.length }));
        }
    });
    
    return words;
}

function updateSpeedReaderWord() {
    if (speedReaderIndex < speedReaderWords.length && speedReaderWords.length > 0) {
        
        let leftWords = [];
        let leftChars = 0;
        for (let i = speedReaderIndex - 1; i >= 0; i--) {
            let item = speedReaderWords[i];
            let len = item.len + 1; // +1 for space
            if (leftChars + len > peripheralChars) break;
            leftWords.unshift(item.html);
            leftChars += len;
        }
        
        let rightWords = [];
        let rightChars = 0;
        for (let i = speedReaderIndex + 1; i < speedReaderWords.length; i++) {
            let item = speedReaderWords[i];
            let len = item.len + 1;
            if (rightChars + len > peripheralChars) break;
            rightWords.push(item.html);
            rightChars += len;
        }
        
        const center = speedReaderWords[speedReaderIndex].html;
        
        const html = `
            <span class="sr-peripheral">${leftWords.join(' ')}</span>
            <span class="sr-center">${center}</span>
            <span class="sr-peripheral">${rightWords.join(' ')}</span>
        `;
        
        document.getElementById('sr-word-display').innerHTML = html;
        
        const total = speedReaderWords.length;
        const progressCount = document.getElementById('sr-progress-count');
        const progressSlider = document.getElementById('sr-progress-slider');
        
        const pct = Math.floor((speedReaderIndex / (total - 1)) * 100);
        progressCount.innerText = `${speedReaderIndex + 1} / ${total} (${pct}%)`;
        progressSlider.value = pct;
        
        if (currentBookId) {
            localStorage.setItem('progress_' + currentBookId, speedReaderIndex);
        }
        
    } else {
        pauseSpeedReader();
        document.getElementById('sr-word-display').innerHTML = "Done";
    }
}

function playSpeedReader() {
    if (speedReaderIndex >= speedReaderWords.length) speedReaderIndex = 0;
    speedReaderPlaying = true;
    document.getElementById('sr-play-pause').innerText = 'Pause';
    const msPerWord = 60000 / speedReaderWpm;
    speedReaderInterval = setInterval(() => {
        updateSpeedReaderWord();
        speedReaderIndex++;
    }, msPerWord);
}

function pauseSpeedReader() {
    speedReaderPlaying = false;
    document.getElementById('sr-play-pause').innerText = 'Play';
    if (speedReaderInterval) {
        clearInterval(speedReaderInterval);
        speedReaderInterval = null;
    }
}

async function loadAllChapters(book) {
    const container = document.getElementById('chapter-content');
    container.innerHTML = '<p>Loading chapters...</p>';
    let allHtml = '';
    
    for (const chapter of book.chapters) {
        const url = `/api/book/${currentBookId}/chapters/${chapter.id}/render?bionic=${bionicEnabled ? '1' : '0'}`;
        try {
            const res = await fetch(url);
            if (res.ok) {
                const html = await res.text();
                const noImagesHtml = html.replace(/<img[^>]*>/gi, '').replace(/<picture[\s\S]*?<\/picture>/gi, '');
                const parser = new DOMParser();
                const doc = parser.parseFromString(noImagesHtml, 'text/html');
                doc.querySelectorAll('svg').forEach(el => el.remove());
                allHtml += `<div id="chapter-${chapter.id}" class="chapter-container">` + doc.body.innerHTML + `</div>`;
            } else {
                console.error('Failed to load chapter');
            }
        } catch (e) {
            console.error('Network error loading chapter', e);
        }
    }
    
    container.innerHTML = allHtml;
    container.scrollTop = 0;
}
