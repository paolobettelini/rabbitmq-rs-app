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

let loaded = 0;
let total = null;

let imageContainer = document.getElementById('images');
let loadMoreButton = document.getElementById('load-more-button');
let imgCounter = document.getElementById('img-counter');

const CHUNK = 5;

let queue = []; // Images queue

postData('/api/total-images')
    .then(json => {
        let container = document.getElementById('img-num');
        total = json['response'];
        container.innerHTML = total;

        load(CHUNK);
        updateButton();
    });

function load(amount) {
    amount = Math.min(loaded + amount, total) - loaded;
    for (let i = loaded + 1; i <= loaded + amount; i++) {
        loadImage(i, i == loaded + amount);
    }
    loaded += amount;
}

loadMoreButton.onclick = () => {
    load(CHUNK);
    updateButton();
};

function updateButton() {
    let amount = Math.min(loaded + CHUNK, total) - loaded;
    if (amount == 0) {
        loadMoreButton.style.display = 'none';
    } else {
        loadMoreButton.innerText = `Load more (${amount})`;
    }

    imgCounter.innerText = `(${loaded}/${total})`;
}

function loadImage(index, lastInChunk) {
    console.log(`Loading image [${index}]`);
    
    let img = document.createElement('img');
    img.src = `/api/image/${index}`;
    queue.push(img);

    if (lastInChunk) {
        img.onload= () => {
            queue.forEach((img, _) => {
                imageContainer.appendChild(img);
            });

            // clear queue
            queue = [];
        }
    }
}