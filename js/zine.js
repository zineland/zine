(function() {

    let urlPreviews = document.querySelectorAll('.url-preview');
    for (let preview of urlPreviews) {
        preview.onclick = (event) => {
            // Ignore the event if the user click the A tag.
            if (event.target.tagName === 'A') return;

            let p = preview.lastElementChild;
            if (p) {
                let a = p.firstChild;
                if (a && a.href) {
                    window.open(a.href);
                }
            }
        };
    }
})();