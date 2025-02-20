import { rootPath } from "./location.mjs";
import User from "./user.mjs";

function nop() {}

async function setupForm() {
    const root = rootPath("me.html");
    if (root !== undefined) {
        console.log(`Root path: "${root}"`);
    } else {
        console.error("login.html is not in the current location");
        return nop;
    }
    const form = document.getElementById("logout-form");
    if (!form) {
        console.error("Logout form not found");
        return nop;
    }
    return function () {
        form.action = `${root}/api/logout`;
    };
}

async function fetchMe() {
    const res = await fetch("/api/me");
    if (!res.ok) {
        console.error("Failed to fetch user information");
        return;
    }
    const data = await res.json();
    const user = User.fromJSON(data);
    if (!user) {
        console.error("Server responded invalid user data");
        return;
    }
    return user;
}

async function setupUser() {
    const user = await fetchMe();
    if (!user) {
        console.error("Failed to fetch user information");
        return nop;
    }
    console.log(`User: ${user.name} (${user.displayId})`);
    const title = `Hello, ${user.name}!`;
    const fieldId = Array.from(document.getElementsByClassName("field-id"));
    const fieldDisplayId = Array.from(document.getElementsByClassName("field-display-id"));
    const fieldName = Array.from(document.getElementsByClassName("field-name"));
    return function () {
        fieldId.forEach((element) => {
            element.textContent = user.id;
        });
        fieldDisplayId.forEach((element) => {
            element.textContent = user.displayId;
        });
        fieldName.forEach((element) => {
            element.textContent = user.name;
        });
        document.title = title;
    };
}

async function setup() {
    const setups = await Promise.all([setupForm(), setupUser()]);
    setups.forEach((setup) => {
        setup();
    });
}

await setup();
