"use strict";

import { hash } from 'frontend'

/* Input elements */
var username_field = document.getElementById('username-temp');
var password_field = document.getElementById('password-temp');
var submit_btn = document.getElementById('submit-button');

/* Status elements */
var username_status = document.getElementById('username-status');
var password_status = document.getElementById('password-status');

/* Form elements */
var form = document.getElementById('form');
var username_form = document.getElementById('username-form');
var password_form = document.getElementById('password-form');

let usernameValid = false;
let passValid = false;

username_field.oninput = () => {
    let regex = /^[a-zA-Z0-9._]{4,20}$/;
    let username = username_field.value;

    if (usernameValid = username.match(regex)) {
        username_status.style.color = 'green';
        username_status.innerHTML = 'Username valid';
    } else {
        username_status.style.color = 'red';
        username_status.innerHTML = username == '' ? '* Required' : 'Invalid username: ^[a-zA-Z0-9._]{4,20}$';
    }

    checkAll();
}

password_field.oninput = () => {
    let regex = /^(?=.*[A-Za-z])(?=.*\d).{8,}$/;
    let password = password_field.value;

    if (passValid = password.match(regex)) {
        password_status.style.color = 'green';
        password_status.innerHTML = 'Password valid';
    } else {
        password_status.style.color = 'red';
        password_status.innerHTML = password == '' ? '* Required' : 'Invalid password: ^(?=.*[A-Za-z])(?=.*\\d).{8,}$';
    }

    checkAll();
}

function checkAll() {
    submit_btn.disabled = !(usernameValid && passValid);
}

submit_btn.onclick = () => {
    username_form.value = username_field.value;
    password_form.value = hash(password_field.value);
    form.submit();
}