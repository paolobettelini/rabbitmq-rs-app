"use strict";

const API_URL = window
    .location
    .toString()
    .substring(0, window
        .location
        .toString()
        .indexOf('/', 8));

Dropzone.autoDiscover = false;

const drop = new Dropzone("#dropzone", {
    url: API_URL + '/api/upload'
});

let statusContainer = document.getElementById('status-container');

drop.on("addedfile", file => {
    if (file.size > 2500000) {
        drop.removeFile(file);
        alert('Maximum file size is 2.5MB')
        return;
    }

    let prefix = document.createTextNode(`File ${file.name}: `);
    let prefixContainer = document.createElement('span');
    
    let valueContainer = document.createElement('span');
    let value = document.createTextNode(sending(0));

    let container = document.createElement('div');

    prefixContainer.appendChild(prefix);
    valueContainer.appendChild(value);
    valueContainer.id = file.upload.uuid;

    container.appendChild(prefixContainer);
    container.appendChild(valueContainer);
    statusContainer.appendChild(container);

    console.log(file);
});

drop.on("uploadprogress", (file, progress, _) => {
    updateStatus(file.upload.uuid, sending(progress));
});

drop.on("error", (file, message) => {
    updateStatus(file.upload.uuid, message);
});

function updateStatus(uuid, content) {
    document.getElementById(uuid).innerText = content;
}

function sending(percentage) {
    return `Sending [${Math.round(percentage)}%]`;
}