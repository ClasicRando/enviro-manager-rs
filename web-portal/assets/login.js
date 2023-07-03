import { fetchApi } from '/assets/utils.js'

/** @type {HTMLFormElement} */
const loginForm = document.getElementById('loginForm');
/** @type {HTMLParagraphElement} */
const errorMessage = document.getElementById('errorMessage');

loginForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    const formData = new FormData(loginForm);
    const response = await fetchApi('/api/login', {
        body: formData,
        method: 'POST',
    });
    console.log(response);
    if (response.success) {
        window.location = '/';
        return;
    }
    errorMessage.innerText = response.message || '';
});
