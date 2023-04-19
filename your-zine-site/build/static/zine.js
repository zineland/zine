(function () {
    const urlPreviews = document.querySelectorAll('.url-preview');
    for (let preview of urlPreviews) {
        preview.onclick = (event) => {
            // Ignore the event if the user click the A tag.
            if (event.target.tagName === 'A') return;

            let a = preview.querySelector('a');
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
                let isLineWrapped = target.getClientRects().length > 1;
                let isSmallViewport = window.innerWidth < 640;
                if (isLineWrapped || isSmallViewport) {
                    inlineCard.style.left = (inlineCard.parentElement.offsetWidth - inlineCard.offsetWidth) / 2 + 'px';
                } else {
                    inlineCard.style.left = Math.max(0, target.offsetLeft + target.offsetWidth / 2 - 190) + 'px';
                }
                inlineCard.style.top = (target.offsetTop - inlineCard.offsetHeight - 15) + 'px';
            };
            link.onmouseleave = () => {
                cardTimeoutId = dismissInTimeout(inlineCard);
            }
        };

        setupMenuList('i18n-menu', 'i18n-list');
        setupMenuList('toc-menu', 'toc-list');
        highlightToc();
    }

    function dismissInTimeout(element) {
        return setTimeout(() => {
            element.classList.add('hidden');
        }, 600);
    }

    function highlightToc() {
        let items = document.querySelectorAll('main .toc-item>a');
        const scrollHandler = entries => {
            // Find the first entry which intersecting and ratio > 0.9 to highlight.
            let entry = entries.find(entry => {
                return entry.isIntersecting && entry.intersectionRatio > 0.9;
            });
            if (!entry) return;

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
        };
        // Set -50% bottom root margin to improve highlight experience.
        const observer = new IntersectionObserver(scrollHandler, { rootMargin: "0% 0% -50% 0%", threshold: 1 });
        items.forEach(item => observer.observe(item));
    }

    function setupMenuList(menuId, listId) {
        let menu = document.getElementById(menuId);
        let list = document.getElementById(listId);
        if (!menu || !list) return;

        menu.onclick = (event) => {
            if (list.classList.contains('hidden')) {
                list.classList.remove('hidden');
            } else {
                list.classList.add('hidden');
            }
            event.stopPropagation();
        };
        list.onclick = (event) => {
            event.stopPropagation();
        };

        document.addEventListener('click', () => {
            list.classList.add('hidden');
        });
    }
})();