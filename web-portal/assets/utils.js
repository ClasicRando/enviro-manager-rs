// import {
//     Grid
// } from 'https://unpkg.com/gridjs?module';

/** @type {(url: RequestInfo | URL, options?: RequestInit | undefined) => Promise<{success: boolean, content: T | null, message: string | null}>} */
export const fetchApi = async (url, options = undefined) => {
    try {
        const response = await fetch(url, options);
        if (!response.ok) {
            try {
                const text = await response.text();
                return {
                    success: false,
                    content: null,
                    message: text,
                }
            } catch (ex) {
                return {
                    success: false,
                    content: null,
                    message: 'Error without a readable body',
                }
            }
        }
        var text;
        try {
            text = await response.text();
            const json = JSON.parse(text);
            if (json.type === 'Success') {
                return {
                    success: true,
                    content: json.data,
                    message: null,
                }
            } else if (json.type === 'Message' || json.type === 'Failure' || json.type === 'Error') {
                return {
                    success: json.type === 'Message',
                    content: null,
                    message: json.data,
                }
            } else {
                return {
                    success: false,
                    content: json,
                    message: 'Unknown response type',
                }
            }
        } catch (error) {
            return {
                success: false,
                content: null,
                message: text || 'Empty or invalid response body',
            }
        }
    } catch (e) {
        return {
            success: false,
            content: null,
            message: e.toString(),
        }
    }
};

/** @type {(classList: DOMTokenList) => Array<string>} */
const filterIconClassList = (classList) => {
    return Array.from(classList.values()).filter(c => c.startsWith('fa-') && c !== 'fa-solid')
};
const getStoredTheme = () => localStorage.getItem('theme');
const setStoredTheme = theme => localStorage.setItem('theme', theme);

const getPreferredTheme = () => {
    const storedTheme = getStoredTheme();
    if (storedTheme) {
        return storedTheme;
    }

    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
};

const setTheme = theme => {
    if (theme === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches) {
        document.documentElement.setAttribute('data-bs-theme', 'dark');
    } else {
        document.documentElement.setAttribute('data-bs-theme', theme);
    }
};

setTheme(getPreferredTheme())

const showActiveTheme = (theme, focus = false) => {
    const themeSwitcher = document.querySelector('#bd-theme');

    if (!themeSwitcher) {
        return;
    }

    const themeSwitcherText = document.querySelector('#bd-theme-text');
    const activeThemeIcon = document.querySelector('#theme-selector');
    const btnToActive = document.querySelector(`[data-bs-theme-value='${theme}']`);
    const classOfActiveBtn = (btnToActive.querySelector('svg') || btnToActive.querySelector('i')).classList;

    document.querySelectorAll('[data-bs-theme-value]').forEach(element => {
        element.classList.remove('active');
        element.setAttribute('aria-pressed', 'false');
    })

    btnToActive.classList.add('active');
    btnToActive.setAttribute('aria-pressed', 'true');
    if (activeThemeIcon.classList.length > 0) {
        const classes = filterIconClassList(activeThemeIcon.classList);
        activeThemeIcon.classList.remove(classes);
    }
    activeThemeIcon.classList.add(filterIconClassList(classOfActiveBtn));
    const themeSwitcherLabel = `${themeSwitcherText.textContent} (${btnToActive.dataset.bsThemeValue})`;
    themeSwitcher.setAttribute('aria-label', themeSwitcherLabel);

    if (focus) {
        themeSwitcher.focus();
    }
}

window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    const storedTheme = getStoredTheme();
    if (storedTheme !== 'light' && storedTheme !== 'dark') {
        setTheme(getPreferredTheme());
    }
});

window.addEventListener('DOMContentLoaded', () => {
    showActiveTheme(getPreferredTheme());

    document.querySelectorAll('[data-bs-theme-value]').forEach(toggle => {
        toggle.addEventListener('click', () => {
            const theme = toggle.getAttribute('data-bs-theme-value');
            setStoredTheme(theme);
            setTheme(theme);
            showActiveTheme(theme, true);
        });
    });
});

/** @type {(element: HTMLElement) => void} */
const clearAllChildren = (element) => {
    while (element.firstChild) {
        element.removeChild(element.lastChild);
    }
};

/** @type {(dec: number) => string} */
const dec2hex = (dec) => dec.toString(16).padStart(2, '0');

/** @type {(len: number) => string} */
const generateId = (len) => {
    const arr = new Uint8Array((len || 40) / 2);
    window.crypto.getRandomValues(arr);
    return Array.from(arr, dec2hex).join('');
};

class Table {
    /** @param {HTMLTableElement} element */
    constructor(element) {
        /** @type {string} */
        this.id = generateId(20);
        /** @type {HTMLTableElement} */
        this.element = element;
        this.element.id = this.id;
        /** @type {string} */
        this.url = element.getAttribute('data-wp-table');
        for (const body of element.tBodies) {
            body.remove();
        }

        const container = document.createElement('div');
        container.classList.add('table-responsive-sm');
        element.parentNode.insertBefore(container, element);
        container.appendChild(element);

        /** @type {HTMLTableSectionElement} */
        this.body = element.createTBody();
        this.body.classList.add('table-group-divider');

        /** @type {string[]} */
        this.keys = this.dataKeysFromHeader(element.tHead);

        /** @type {HTMLTableRowElement} */
        this.loadingRow = document.createElement('tr');
        const cell = document.createElement('td');
        cell.colSpan = this.keys.length;
        cell.innerText = 'Loading...';
        this.loadingRow.appendChild(cell);
    }

    /** @type {() => Array<string>} */
    dataKeysFromHeader() {
        const keys = [];
        for (const column of this.element.tHead.querySelectorAll('th')) {
            const key = column.getAttribute('data-wp-table-key') || column.textContent;
            keys.push(key);
        }
        return keys;
    }

    setLoading() {
        clearAllChildren(this.body);
        this.body.appendChild(this.loadingRow);
    }

    /** @type {(message: string | undefined) => void} */
    unsetLoading(message) {
        this.body.removeChild(this.loadingRow);
        if (typeof message === "string") {
            const messageRow = document.createElement('tr');
            const cell = document.createElement('td');
            cell.colSpan = this.keys.length;
            cell.innerText = message;
            messageRow.appendChild(cell);
            this.body.appendChild(messageRow);
        }
    }

    addRow(data) {
        const row = document.createElement('tr');
        for (const key of this.keys) {
            const cell = document.createElement('td');
            cell.innerText = data[key] || '';
            row.appendChild(cell);
        }
        this.body.appendChild(row);
    }

    async RefreshData() {
        this.setLoading();
        const response = await fetchApi(this.url);
        if (!response.success) {
            console.error('Unsuccessful api fetch', response);
            this.unsetLoading('Unsuccessful api fetch');
            return;
        }
        if (typeof response.message === 'string') {
            console.error('Expected data but got a message', response.message);
            this.unsetLoading('Expected data but got a message');
            return;
        }
        this.unsetLoading();

        const data = response.content;
        const items = Array.isArray(data) ? data : [data];
        items.forEach(item => this.addRow(item));
    }
}

/** @type {(element: HTMLTableElement) => Promise<Table>} */
const buildTable = async (element) => {
    const table = new Table(element);
    await table.RefreshData();
    return table;
};

window.addEventListener('DOMContentLoaded', async () => {
    /** @type {Array<Promise<Table>>} */
    const builders = [];
    for (const table of document.querySelectorAll('[data-wp-table]')) {
        builders.push(buildTable(table));
    };
    await Promise.allSettled(builders);
    console.log('Done building tables');
});
