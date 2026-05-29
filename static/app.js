// Minimal Vanilla JS Frontend

let currentBookId = null;
let currentBookData = null;
let bionicEnabled = false;
let chapterBoundaries = [];
let peripheralChars = 20;
let peripheralBrightness = 0.5;

function saveSettingsToCookie() {
    const settings = {
        themeDark: document.getElementById('theme-toggle').checked,
        fontSize: document.getElementById('font-size').value,
        fontWeight: document.getElementById('font-weight').value,
        peripheralChars: document.getElementById('peripheral-chars').value,
        peripheralBrightness: document.getElementById('peripheral-brightness').value,
        bionic: document.getElementById('bionic-toggle').checked,
        focusColor: document.getElementById('focus-color').value,
        fontFamily: document.getElementById('font-family').value,
        customFont: document.getElementById('custom-font').value
    };
    document.cookie = "fsr_settings=" + encodeURIComponent(JSON.stringify(settings)) + "; path=/; max-age=31536000";
}

function loadSettingsFromCookie() {
    const match = document.cookie.match(new RegExp('(^| )fsr_settings=([^;]+)'));
    if (match) {
        try {
            const settings = JSON.parse(decodeURIComponent(match[2]));
            if (settings.themeDark) {
                document.getElementById('theme-toggle').checked = true;
                document.body.classList.add('theme-dark');
            }
            if (settings.fontSize) {
                document.getElementById('font-size').value = settings.fontSize;
                document.documentElement.style.setProperty('--user-font-size', `${settings.fontSize}rem`);
            }
            if (settings.fontWeight) {
                document.getElementById('font-weight').value = settings.fontWeight;
                document.documentElement.style.setProperty('--user-font-weight', settings.fontWeight);
            }
            if (settings.peripheralChars) {
                document.getElementById('peripheral-chars').value = settings.peripheralChars;
                peripheralChars = parseInt(settings.peripheralChars, 10);
            }
            if (settings.peripheralBrightness) {
                document.getElementById('peripheral-brightness').value = settings.peripheralBrightness;
                peripheralBrightness = parseFloat(settings.peripheralBrightness);
                document.documentElement.style.setProperty('--peripheral-opacity', peripheralBrightness);
            }
            if (settings.bionic !== undefined) {
                document.getElementById('bionic-toggle').checked = settings.bionic;
                bionicEnabled = settings.bionic;
            }
            if (settings.focusColor) {
                document.getElementById('focus-color').value = settings.focusColor;
                document.documentElement.style.setProperty('--focus-color', settings.focusColor);
            }
            if (settings.fontFamily) {
                document.getElementById('font-family').value = settings.fontFamily;
                if (settings.fontFamily === 'custom') {
                    document.getElementById('custom-font-label').style.display = 'flex';
                    if (settings.customFont) {
                        document.getElementById('custom-font').value = settings.customFont;
                        document.documentElement.style.setProperty('--user-font-family', settings.customFont);
                    }
                } else {
                    document.documentElement.style.setProperty('--user-font-family', settings.fontFamily);
                }
            }
            return true;
        } catch(e) { console.error("Error loading settings from cookie", e); return false; }
    }
    return false;
}

document.addEventListener('DOMContentLoaded', async () => {
    const settingsLoaded = loadSettingsFromCookie();
    if (!settingsLoaded) {
        // Fallback to localStorage
        const savedFocusColor = localStorage.getItem('focusColor');
        if (savedFocusColor) {
            document.documentElement.style.setProperty('--focus-color', savedFocusColor);
            const fcInput = document.getElementById('focus-color');
            if (fcInput) fcInput.value = savedFocusColor;
        }
        
        const savedFontFamilySelect = localStorage.getItem('fontFamilySelect');
        if (savedFontFamilySelect) {
            const ffSelect = document.getElementById('font-family');
            if (ffSelect) ffSelect.value = savedFontFamilySelect;
            if (savedFontFamilySelect === 'custom') {
                const cfLabel = document.getElementById('custom-font-label');
                if (cfLabel) cfLabel.style.display = 'flex';
            }
        }
        
        const savedCustomFontValue = localStorage.getItem('customFontValue');
        if (savedCustomFontValue) {
            const cfInput = document.getElementById('custom-font');
            if (cfInput) cfInput.value = savedCustomFontValue;
        }
        
        const savedUserFontFamily = localStorage.getItem('userFontFamily');
        if (savedUserFontFamily) {
            document.documentElement.style.setProperty('--user-font-family', savedUserFontFamily);
        }
    }

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
        
        document.getElementById('reader-container').classList.remove('hidden');
        document.getElementById('library-view').classList.add('hidden');
        
        document.getElementById('book-title').innerText = book.title;
        
        const tocContainer = document.getElementById('toc');
        const srChapterSelect = document.getElementById('sr-chapter-select');
        srChapterSelect.innerHTML = '';
        book.chapters.forEach((chapter, index) => {
            const link = document.createElement('a');
            link.href = "#";
            const chapterTitle = chapter.title || `Chapter ${chapter.order + 1}`;
            link.innerText = chapterTitle;
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
            
            const option = document.createElement('option');
            option.value = index;
            option.innerText = chapterTitle;
            srChapterSelect.appendChild(option);
        });
        
        // 2. Setup event listeners
        document.getElementById('bionic-toggle').addEventListener('change', (e) => {
            bionicEnabled = e.target.checked;
            saveSettingsToCookie();
            if (currentBookData) loadAllChapters(currentBookData);
        });
        
        document.getElementById('focus-color').addEventListener('input', (e) => {
            document.documentElement.style.setProperty('--focus-color', e.target.value);
            localStorage.setItem('focusColor', e.target.value);
            saveSettingsToCookie();
        });

        document.getElementById('font-family').addEventListener('change', (e) => {
            const val = e.target.value;
            if (val === 'custom') {
                document.getElementById('custom-font-label').style.display = 'flex';
                const customVal = document.getElementById('custom-font').value;
                if (customVal) {
                    document.documentElement.style.setProperty('--user-font-family', customVal);
                    localStorage.setItem('userFontFamily', customVal);
                }
            } else {
                document.getElementById('custom-font-label').style.display = 'none';
                document.documentElement.style.setProperty('--user-font-family', val);
                localStorage.setItem('userFontFamily', val);
            }
            localStorage.setItem('fontFamilySelect', val);
            saveSettingsToCookie();
        });

        document.getElementById('custom-font').addEventListener('input', (e) => {
            const val = e.target.value;
            if (val) {
                document.documentElement.style.setProperty('--user-font-family', val);
                localStorage.setItem('userFontFamily', val);
                localStorage.setItem('customFontValue', val);
                saveSettingsToCookie();
            }
        });
        
        document.getElementById('theme-toggle').addEventListener('change', (e) => {
            if (e.target.checked) document.body.classList.add('theme-dark');
            else document.body.classList.remove('theme-dark');
            saveSettingsToCookie();
        });

        document.getElementById('font-size').addEventListener('input', (e) => {
            document.documentElement.style.setProperty('--user-font-size', `${e.target.value}rem`);
            saveSettingsToCookie();
        });

        document.getElementById('font-weight').addEventListener('input', (e) => {
            document.documentElement.style.setProperty('--user-font-weight', e.target.value);
            saveSettingsToCookie();
        });

        document.getElementById('peripheral-chars').addEventListener('input', (e) => {
            peripheralChars = parseInt(e.target.value, 10);
            saveSettingsToCookie();
            if (!speedReaderPlaying) updateSpeedReaderWord();
        });

        document.getElementById('peripheral-brightness').addEventListener('input', (e) => {
            peripheralBrightness = parseFloat(e.target.value);
            document.documentElement.style.setProperty('--peripheral-opacity', peripheralBrightness);
            saveSettingsToCookie();
        });
        
        document.getElementById('toggle-sidebar').addEventListener('click', (e) => {
            const sidebar = document.getElementById('sidebar');
            const isHidden = sidebar.classList.contains('hidden');
            if (isHidden) {
                sidebar.classList.remove('hidden');
                e.currentTarget.setAttribute('aria-expanded', 'true');
            } else {
                sidebar.classList.add('hidden');
                e.currentTarget.setAttribute('aria-expanded', 'false');
            }
        });

        const settingsPanel = document.getElementById('settings-panel');
        let currentSettingsTrigger = null;

        function updateSettingsTrigger(e) {
            const isDifferentTrigger = currentSettingsTrigger !== e.currentTarget;
            currentSettingsTrigger = e.currentTarget;
            
            if (currentSettingsTrigger.id === 'sr-settings') {
                document.getElementById('speed-reader-overlay').appendChild(settingsPanel);
            } else {
                document.getElementById('app').appendChild(settingsPanel);
            }
            
            if (settingsPanel.matches(':popover-open')) {
                if (isDifferentTrigger) {
                    repositionSettings();
                } else {
                    settingsPanel.hidePopover();
                }
            } else {
                settingsPanel.showPopover();
            }
        }

        document.getElementById('toggle-settings').addEventListener('click', updateSettingsTrigger);
        document.getElementById('sr-settings').addEventListener('click', updateSettingsTrigger);

        function repositionSettings() {
            if (settingsPanel.matches(':popover-open') && currentSettingsTrigger) {
                const rect = currentSettingsTrigger.getBoundingClientRect();
                settingsPanel.style.top = (rect.bottom + 5) + 'px';
                let left = rect.right - settingsPanel.offsetWidth;
                if (left < 0) left = 10;
                settingsPanel.style.left = left + 'px';
            }
        }

        settingsPanel.addEventListener('toggle', (e) => {
            if (e.newState === 'open' && currentSettingsTrigger) {
                currentSettingsTrigger.setAttribute('aria-expanded', 'true');
                repositionSettings();
            } else {
                if (currentSettingsTrigger) {
                    currentSettingsTrigger.setAttribute('aria-expanded', 'false');
                }
            }
        });

        document.getElementById('close-settings').addEventListener('click', () => {
            settingsPanel.hidePopover();
        });

        window.addEventListener('resize', repositionSettings);
        window.addEventListener('scroll', repositionSettings, { capture: true });

        // Speed Reader logic
        document.getElementById('start-speed-read').addEventListener('click', (e) => {
            const dialog = document.getElementById('speed-reader-overlay');
            e.currentTarget.setAttribute('aria-expanded', 'true');
            dialog.showModal();
            const playBtn = document.getElementById('sr-play-pause');
            if (playBtn) playBtn.focus();

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
            document.getElementById('speed-reader-overlay').close();
        });

        document.getElementById('speed-reader-overlay').addEventListener('close', () => {
            document.getElementById('start-speed-read').setAttribute('aria-expanded', 'false');
            pauseSpeedReader();
        });

        document.getElementById('sr-chapter-select').addEventListener('change', (e) => {
            const idx = parseInt(e.target.value, 10);
            if (chapterBoundaries[idx] !== undefined) {
                speedReaderIndex = chapterBoundaries[idx];
                updateSpeedReaderWord();
            }
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
        
        let currentChapterIndex = 0;
        for (let i = chapterBoundaries.length - 1; i >= 0; i--) {
            if (speedReaderIndex >= chapterBoundaries[i]) {
                currentChapterIndex = i;
                break;
            }
        }
        document.getElementById('sr-chapter-select').value = currentChapterIndex;
        
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
