"use strict";

const API_URL = window
    .location
    .toString()
    .substring(0, window
        .location
        .toString()
        .indexOf('/', 8));

Dropzone.autoDiscover = false;
Dropzone.parallelUploads = 5;

const drop = new Dropzone("#dropzone", {
    url: API_URL + '/api/upload',
});

let statusContainer = document.getElementById('status-container');

// Avoid multiple consecutive alerts
let lastUploadedFolder = undefined;

drop.on("addedfile", file => {
    // Folder upload prevention
    if (file.fullPath !== undefined) {
        let sep1 = file.fullPath.includes("/");
        let sep2 = file.fullPath.includes("\\");
        
        if (sep1 || sep2) {
            drop.removeFile(file);
            
            if (!file.fullPath.startsWith(lastUploadedFolder)) {
                lastUploadedFolder = file.fullPath.substr(0, file.fullPath.indexOf(sep1 ? "/" : "\\"));
                alert("Do not upload files within a directory but rather the files itself.");
            }
            
            return;
        }
    }

    // Check content type
    if (!file.type.startsWith("image")) {
        drop.removeFile(file);
        alert('This file is not an image');
        return;
    }

    // Big file upload prevention
    if (file.size > 2500000) {
        drop.removeFile(file);
        alert('Maximum file size is 2.5MB');
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
});

drop.on("uploadprogress", (file, progress, _) => {
    updateStatus(file.upload.uuid, sending(progress));
});

drop.on("error", (file, _) => {
    updateStatus(file.upload.uuid, "An error has occured");
});

drop.on("success", (file, msg) => {
    updateStatus(file.upload.uuid, msg["response"]);
});

function updateStatus(uuid, content) {
    document.getElementById(uuid).innerText = content;
}

function sending(percentage) {
    return `Sending [${Math.round(percentage)}%]`;
}