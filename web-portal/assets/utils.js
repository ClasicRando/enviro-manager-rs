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
window.toggleDisplay = (element) => {
    if (element.classList.contains('d-none')) {
        element.classList.remove('d-none');
    } else {
        element.classList.add('d-none');
    }
};

/** @type {(element: HTMLElement) => void} */
window.selectTab = (element) => {
    const tabList = element.parentElement.parentElement;
    tabList.querySelectorAll('button.nav-link.active').forEach(btn => {
        btn.classList.remove('active');
    });
    element.classList.add('active');
};

window.addEventListener('DOMContentLoaded', () => {
    const navScroll = document.querySelector('#mainNavBar');
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

/**
 * 
 * @param {string} body 
 */
function createToast(body) {
    const container = document.getElementById('toasts');
    if (container === null) {
        return;
    }
    const toast = document.createElement('div');
    toast.classList.add('toast', 'fade', 'show');
    toast.setAttribute('role', 'alert');
    toast.setAttribute('aria-live', 'assertive');
    toast.setAttribute('aria-atomic', 'true');
    const header = document.createElement('div');
    toast.appendChild(header);
    header.classList.add('toast-header');
    const img = document.createElement('img');
    header.appendChild(img);
    img.src = '/assets/bell_icon.png';
    img.width = 20;
    img.classList.add('me-1');
    const title = document.createElement('strong');
    header.appendChild(title);
    title.textContent = 'EnviroManager';
    title.classList.add('me-auto');
    const dismiss = document.createElement('button');
    header.appendChild(dismiss);
    dismiss.type = 'button';
    dismiss.classList.add('btn-close');
    dismiss.setAttribute('data-bs-dismiss', 'toast');
    dismiss.setAttribute('aria-label', 'Close');
    const content = document.createElement('div');
    toast.appendChild(content);
    content.classList.add('toast-body');
    content.textContent = body;
    container.appendChild(toast);
}

window.addEventListener('DOMContentLoaded', () => {
    setInterval(() => {
        for (const element of document.querySelectorAll('.request-toast')) {
            const body = element.textContent;
            createToast(body);
        }
    }, 500);
});
