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
        clearAllChildren(this.element);

        const container = document.createElement('div');
        container.classList.add('table-responsive-sm');
        element.parentNode.insertBefore(container, element);
        container.appendChild(element);

        /** @type {HTMLTableRowElement} */
        this.loadingRow = document.createElement('tr');
        const cell = document.createElement('td');
        cell.innerText = 'Loading...';
        this.loadingRow.appendChild(cell);

        /** @type {HTMLTableSectionElement | null} */
        this.body = null;
        /** @type {HTMLTableSectionElement | null} */
        this.header = null;
    }

    /** 
     * @returns {void}
     */
    resetInnerContents() {
        clearAllChildren(this.element);
        this.header = this.element.createTHead();
        this.body = this.element.createTBody();
        this.body.classList.add('table-group-divider');
    }

    /** 
     * @returns {void}
     */
    setLoading() {
        this.resetInnerContents();
        this.body.appendChild(this.loadingRow);
    }

    /** 
     * @param {string | undefined} message
     * @returns {void}
     */
    unsetLoading(message) {
        this.body.removeChild(this.loadingRow);
        if (typeof message === "string") {
            const messageRow = document.createElement('tr');
            const cell = document.createElement('td');
            cell.innerText = message;
            messageRow.appendChild(cell);
            this.body.appendChild(messageRow);
        }
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

        /** @type {{caption: string, columns: {key: string, title: string}[], data: T[], details: {key: string, columns: {key: string, title: string}[]} | null}} */
        const tableData = response.content;
        const caption = document.createElement('caption');
        caption.textContent = tableData.caption;
        this.element.insertBefore(caption, this.header);
        const headerRow = document.createElement('tr');
        this.header.appendChild(headerRow);
        if (typeof tableData.details === 'string') {
            addCellWithText(headerRow, 'Details');
        }
        tableData.columns.forEach(column => addCellWithText(headerRow, column.title));
        tableData.data.forEach(item => addRow(this.body, item, tableData.columns, tableData.details));
    }
}

/**
 * @param {T[]} data
 * @param {{key: string, title: string}[]} columns
 * @param {number} columnCount
 * @returns {HTMLTableRowElement}
 */
const createDetailsRow = (data, columns, columnCount) => {
    const row = document.createElement('tr');
    const cell = document.createElement('td');
    cell.colSpan = columnCount;
    row.appendChild(cell);

    if (data.length === 0) {
        return row;
    }

    const detailsTable = document.createElement('table');
    cell.appendChild(detailsTable);
    detailsTable.classList.add('table', 'mb-0');

    const header = detailsTable.createTHead();
    const headerRow = document.createElement('tr');
    header.appendChild(headerRow);
    columns.forEach(column => addCellWithText(headerRow, column.title));

    const body = detailsTable.createTBody();
    data.forEach(item => addRow(body, item, columns, null));

    return row;
};

/** @type {(row: HTMLTableSectionElement, text: string | null)} */
const addCellWithText = (row, text) => {
    const cell = row.tagName === 'thead' ? document.createElement('th') : document.createElement('td');
    cell.textContent = text;
    row.appendChild(cell);
};

/** @type {(body: HTMLTableSectionElement, item: T, columns: {key: string, title: string}[], details: {key: string, columns: {key: string, title: string}[]} | null) => void})} */
const addRow = (body, item, columns, details) => {
    const row = document.createElement('tr');
    body.appendChild(row);
    if (details !== null) {
        const cell = document.createElement('td');
        const button = document.createElement('button');
        button.classList.add('btn', 'btn-primary');
        let opened = false;
        /** @type {HTMLTableRowElement | null} */
        let detailsRow = null;
        button.addEventListener('click', () => {
            opened = !opened;
            if (opened) {
                detailsRow = createDetailsRow(item[details.key], details.columns, columns.length + 1);
                row.after(detailsRow);
            } else {
                detailsRow.remove();
            }
        });
        const symbol = document.createElement('i');
        symbol.classList.add('fa-solid', 'fa-plus');
        button.appendChild(symbol);
        cell.appendChild(button);
        row.appendChild(cell);
    }
    for (const col of columns) {
        addCellWithText(row, item[col.key] || '');
    }
};

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
});

window.addEventListener('DOMContentLoaded', () => {
    const navScroll = document.querySelector('#navbarScroll');
    for (const navLink of navScroll.querySelectorAll('a.nav-link')) {
        navLink.classList.remove('active');
        if (navLink.href.endsWith("/")) {
            continue;
        }
        if (window.location.href.startsWith(navLink.href)) {
            navLink.classList.add('active');
        }
    }
});
