/** @type {(url: RequestInfo | URL, options?: RequestInit | undefined) => Promise<{success: boolean, redirect: string | undefined, content: T | string | undefined}>} */
export const fetchApi = async (url, options = undefined) => {
    try {
        const response = await fetch(url, options);
        console.log(response);
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
        try {
            const json = await response.json();
            return {
                success: true,
                content: json,
            }
        } catch (error) {
            return {
                success: false,
                content: "Found OK response with no json body",
            }
        }
    } catch (e) {
        return {
            success: false,
            content: e.toString(),
        }
    }
};
