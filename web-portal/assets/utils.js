/** @type {(url: RequestInfo | URL, options?: RequestInit | undefined) => Promise<{success: boolean, redirect: string | undefined, content: T | string | undefined}>} */
export const fetchApi = async (url, options = undefined) => {
    try {
        const response = await fetch(url, options);
        if (!response.ok) {
            try {
                const text = await response.text();
                return {
                    success: false,
                    content: text,
                }
            } catch (ex) {
                return {
                    success: false,
                    content: "Error without a readable body",
                }
            }
        }
        if (response.redirected) {
            return {
                success: true,
                redirect: response.url,
            }
        }
        var text;
        try {
            text = await response.text();
            const json = JSON.parse(text);
            return {
                success: true,
                content: json || text,
            }
        } catch (error) {
            return {
                success: false,
                content: text || "Empty or invalid response body",
            }
        }
    } catch (e) {
        return {
            success: false,
            content: e.toString(),
        }
    }
};

/** @type {(classList: DOMTokenList) => Array<string>} */
const filterIconClassList = (classList) => {
    return Array.from(classList.values()).filter(c => c.startsWith("fa-") && c !== "fa-solid")
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
    const btnToActive = document.querySelector(`[data-bs-theme-value="${theme}"]`);
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

