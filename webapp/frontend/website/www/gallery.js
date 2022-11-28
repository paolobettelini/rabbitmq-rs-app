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

var loaded = 0;
var total = null;

var imageContainer = document.getElementById('images');
var loadMoreButton = document.getElementById('load-more-button');

postData('/api/total-images')
    .then(json => {
        let container = document.getElementById('img-num');
        total = json['response'];
        container.innerHTML = total;

        load(5);
    });

function load(amount) {
    amount = Math.min(loaded + amount, total) - loaded;
    for (let i = loaded + 1; i <= loaded + amount; i++) {
        loadImage(i);
    }
    loaded += amount;
}

loadMoreButton.onclick = () => load(5);

function loadImage(index) {
    console.log(`Loading image [${index}]`);
    
    let img = document.createElement('img');
    img.src = `/api/image/${index}`;
    
    imageContainer.appendChild(img);
}