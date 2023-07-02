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
