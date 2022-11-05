// import { hash } from './common.js'
import { hash } from 'frontend'

console.log(hash('ciaoo'));

/* Input elements */
var username_field = document.getElementById('username-temp');
var email_field = document.getElementById('email-temp');
var password_field = document.getElementById('password-temp');
var submit_btn = document.getElementById('submit-button');

/* Status elements */
var username_status = document.getElementById('username-status');
var email_status = document.getElementById('email-status');
var password_status = document.getElementById('password-status');

/* Form elements */
var form = document.getElementById('form');
var username_form = document.getElementById('username-form');
var email_form = document.getElementById('email-form');
var password_form = document.getElementById('password-form');

let emailValid = false;
let usernameValid = false;
let passValid = false;

function checkUsername() {
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

function checkEmail() {
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

function checkPassword() {
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
    submit_btn.disabled = !(emailValid && usernameValid && passValid);
}

function submit() {
    let a = hash(password_field.value);
    console.log(a);

    return;
    username_form.value = username_field.value;
    password_form.value = password_field.value;
    email_form.value = email_field.value;
    form.submit();
}