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

const CLASS_NAME_SHOW = 'show';
const ATTRIBUTE_NAME_POPPER = 'data-bs-popper';

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
    for (const dropdownToggle of document.querySelectorAll('.nav-link.dropdown-toggle')) {
        new DropDown(dropdownToggle);
    }
});

window.addEventListener('DOMContentLoaded', () => {
    setInterval(() => {
        for (const element of document.querySelectorAll('div.toast.show')) {
            let autoDismissTimeout = 500;
            try {
                autoDismissTimeout = Number.parseInt(element.getAttribute('data-em-toast-dismiss'));
            } catch (ex) {
                console.error(ex);
            }
            setTimeout(() => {
                element.remove();
            }, autoDismissTimeout);
        }
    }, 1000);
});

/** @type {(element: HTMLElement) => void} */
window.closeModal = (element) => {
    const container = document.getElementById('modals');
    const modalId = element.getAttribute('data-em-modal');
    if (!modalId) {
        console.warn('Could not find modal to close');
        return;
    }
    const modal = document.getElementById(modalId);
    const modalBackdrop = document.getElementById(`${modalId}-backdrop`);

    modal.classList.remove(CLASS_NAME_SHOW);
    modalBackdrop.classList.remove(CLASS_NAME_SHOW);

    setTimeout(() => {
        container.removeChild(modal);
        container.removeChild(modalBackdrop);
    }, 200);
}

/** @type {(element: HTMLElement) => void} */
window.closeToast = (element) => {
    const container = document.getElementById('toasts');
    const toast = element.parentElement.parentElement;

    toast.classList.remove(CLASS_NAME_SHOW);

    setTimeout(() => {
        container.removeChild(toast);
    }, 200);
}

class DropDown {
    /** @param {HTMLElement} element */
    constructor(element) {
        /** @type {HTMLElement} */
        this.element = element;
        /** @type {HTMLUListElement} */
        this.menu = element.parentElement.querySelector('ul.dropdown-menu');
        this.element.addEventListener('click', (e) => {
            e.preventDefault();
            this.toggle();
        });
        for (const item of this.menu.querySelectorAll('.dropdown-item')) {
            item.addEventListener('click', (e) => {
                e.preventDefault();
                this.toggle();
            });
        }
    }

    toggle() {
        if (this.isShown) {
            this._hide();
        } else {
            this._show();
        }
    }

    get isShown() {
        return this.element.classList.contains(CLASS_NAME_SHOW);
    }

    _show() {
        this.element.classList.add(CLASS_NAME_SHOW);
        this.menu.classList.add(CLASS_NAME_SHOW);
        this.menu.setAttribute(ATTRIBUTE_NAME_POPPER, 'static');
    }

    _hide() {
        this.element.classList.remove(CLASS_NAME_SHOW);
        this.menu.classList.remove(CLASS_NAME_SHOW);
        this.menu.removeAttribute(ATTRIBUTE_NAME_POPPER);
    }
}
