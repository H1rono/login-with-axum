import { rootPath } from "./location.mjs";

function setupForm() {
    const root = rootPath('login.html');
    if (root !== undefined) {
        console.log(`Root path: "${root}"`);
    } else {
        console.error('login.html is not in the current location');
        return;
    }
    const form = document.getElementById('login-form');
    if (!form) {
        console.error('Login form not found');
        return;
    }
    form.action = `${root}/api/login`;
}

setupForm();
