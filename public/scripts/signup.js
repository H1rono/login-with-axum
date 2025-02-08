import { rootPath } from "./location.mjs";

function setupForm() {
    const root = rootPath('signup.html');
    if (root !== undefined) {
        console.log(`Root path: "${root}"`);
    } else {
        console.error('signup.html is not in the current location');
        return;
    }
    const form = document.getElementById('signup-form');
    if (!form) {
        console.error('Signup form not found');
        return;
    }
    form.action = `${root}/api/register`;
}

setupForm();
