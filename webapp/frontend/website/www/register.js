"use strict";

import { hash } from 'frontend'

/* Input elements */
var username_field = document.getElementById('username-temp');
var email_field = document.getElementById('email-temp');
var password1_field = document.getElementById('password1-temp');
var password2_field = document.getElementById('password2-temp');
var submit_btn = document.getElementById('submit-button');

/* Status elements */
var username_status = document.getElementById('username-status');
var email_status = document.getElementById('email-status');
var password1_status = document.getElementById('password1-status');
var password2_status = document.getElementById('password2-status');

/* Form elements */
var form = document.getElementById('form');
var username_form = document.getElementById('username-form');
var email_form = document.getElementById('email-form');
var password_form = document.getElementById('password-form');

let emailValid = false;
let usernameValid = false;
let pass1Valid = false;
let pass2Valid = false;

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

email_field.oninput = () => {
    let regex = /^\w+([\.-]?\w+)*@\w+([\.-]?\w+)*(\.\w{2,3})+$/;
    let email = email_field.value;

    if (emailValid = email.match(regex)) {
        email_status.style.color = 'green';
        email_status.innerHTML = 'Email valid';
    } else {
        email_status.style.color = 'red';
        email_status.innerHTML = email == '' ? '* Required' : 'Invalid email';
    }

    checkAll();
}

password1_field.oninput = () => {
    let regex = /^(?=.*[A-Za-z])(?=.*\d).{8,}$/;
    let password = password1_field.value;

    if (pass1Valid = password.match(regex)) {
        password1_status.style.color = 'green';
        password1_status.innerHTML = 'Password valid';
    } else {
        password1_status.style.color = 'red';
        password1_status.innerHTML = password == '' ? '* Required' : 'Invalid password: ^(?=.*[A-Za-z])(?=.*\\d).{8,}$';
    }

    checkAll();
}

password2_field.oninput = () => {
    let password1 = password1_field.value;
    let password2 = password2_field.value;

    if (pass2Valid = password1 == password2) {
        password2_status.style.color = 'green';
        password2_status.innerHTML = 'Password matches';
    } else {
        password2_status.style.color = 'red';
        password2_status.innerHTML = password2 == '' ? '* Required' : 'Passwords do not match';
    }

    checkAll();
}

function checkAll() {
    submit_btn.disabled = !(emailValid && usernameValid && pass1Valid && pass2Valid);
}

submit_btn.onclick = () => {
    username_form.value = username_field.value;
    password_form.value = hash(password1_field.value);
    email_form.value = email_field.value;
    form.submit();
}