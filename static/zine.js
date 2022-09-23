(function () {
    const urlPreviews = document.querySelectorAll('.url-preview');
    for (let preview of urlPreviews) {
        preview.onclick = (event) => {
            // Ignore the event if the user click the A tag.
            if (event.target.tagName === 'A') return;

            let a = preview.lastElementChild;
            if (a && a.href) {
                window.open(a.href);
            }
        };
    }

    mediumZoom(document.querySelectorAll('.prose p>img'), {
        margin: 24,
        background: '#000C',
    });

    let inlineCard = document.querySelector('.inline-link-card');
    let cardTimeoutId;
    if (inlineCard) {
        inlineCard.onmouseover = () => {
            clearTimeout(cardTimeoutId);
        }
        inlineCard.onmouseleave = () => {
            dismissInTimeout(inlineCard);
        };

        const inlineLinks = document.querySelectorAll('.inline-link');
        for (let link of inlineLinks) {
            link.onmouseover = (event) => {
                clearTimeout(cardTimeoutId);
                const target = event.target;
                inlineCard.querySelector('.inline-link-url').setAttribute('href', target.getAttribute('data-url'));
                inlineCard.querySelector('.inline-link-image').setAttribute('src', target.getAttribute('data-image'));
                inlineCard.querySelector('.inline-link-title').textContent = target.getAttribute('data-title');

                inlineCard.classList.remove("hidden");
                // Align card top center above link.
                inlineCard.style.left = Math.max(3, target.offsetLeft + target.offsetWidth / 2 - 190) + 'px';
                inlineCard.style.top = (target.offsetTop - 275) + 'px';
            };
            link.onmouseleave = () => {
                cardTimeoutId = dismissInTimeout(inlineCard);
            }
        };

        setupArticleToc();
    }

    function dismissInTimeout(element) {
        return setTimeout(() => {
            element.classList.add('hidden');
        }, 600);
    }

    function highlightToc() {
        let items = document.querySelectorAll('main .toc-item>a');
        const scrollHandler = entries => {
            entries.forEach(entry => {
                if (entry.intersectionRatio > 0) {
                    document.querySelectorAll("#toc-list div").forEach((item) => {
                        item.classList.remove("toc-active");
                    });

                    let url = new URL(entry.target.href);
                    let link = document.querySelector(`#toc-list a[href$="${decodeURIComponent(url.hash)}"]`)
                    if (link) {
                        let target = link.querySelector('div');
                        target.classList.add("toc-active");
                        target.scrollIntoView({ behavior: "auto", block: "nearest" });
                    }
                }
            });
        };
        const observer = new IntersectionObserver(scrollHandler);
        items.forEach(item => observer.observe(item));
    }

    function setupArticleToc() {
        let tocMenu = document.getElementById('toc-menu');
        let tocList = document.getElementById('toc-list');
        if (!tocMenu || !tocList) return;

        tocMenu.onclick = (event) => {
            if (tocList.classList.contains('hidden')) {
                tocList.classList.remove('hidden');
            } else {
                tocList.classList.add('hidden');
            }
            event.stopPropagation();
        };
        tocList.onclick = (event) => {
            event.stopPropagation();
        };

        document.addEventListener('click', () => {
            tocList.classList.add('hidden');
        });

        highlightToc();
    }
})();