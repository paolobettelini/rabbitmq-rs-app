"use strict";

const API_URL = window
    .location
    .toString()
    .substring(0, window
        .location
        .toString()
        .indexOf('/', 8));

async function postData(url = '', data = {}) {
    url = API_URL + url;

    const response = await fetch(url, {
        method: 'POST',
        cache: 'no-cache',
        headers: {
            'Content-Type': 'application/json'
        },
        referrerPolicy: 'no-referrer',
        body: JSON.stringify(data)
    });
    return await response.json();
}

postData('/api/total-images')
    .then(json => {
        let container = document.getElementById('img-num');
        let total = json['response'];
        let text = document.createTextNode(total);
        container.appendChild(text);
    });