const tagsFor = item => Array.from(item.querySelectorAll('.tag')).map(tag => tag.textContent);
const select = selector => Array.from(document.querySelectorAll(selector));

function filterByTag(requested) {
    select('li').forEach(item => {
        const tags = tagsFor(item);
        const show = requested.every(tag => tags.includes(tag));
        item.style.display = show ? '' : 'none';
    });

    // Emphasize selected tags.
    requested.forEach(tag => {
        select(`.tag_${tag}`).forEach(tag => tag.style.fontWeight = 'bold');
    });
}

// Return true if every item tagged with tag2 is also tagged with tag1.
function isSuperset(tag1, tag2) {
    return select('li').every(item => {
        const tags = tagsFor(item);
        return !tags.includes(tag2) || tags.includes(tag1);
    });
}

function isDisjoint(tag1, tag2) {
    return select('li').every(item => {
        const tags = tagsFor(item);
        return !tags.includes(tag2) || !tags.includes(tag1);
    });
}

function resetSelection() {
    select('li').forEach(item => item.style.display = ''); // show hidden
    select('.tag').forEach(item => item.style.fontWeight = ''); // de-emphasize
    select('.site').forEach(site => site.style.fontWeight = '');
}

function filterBySite(requested) {
    const siteString = `(${requested})`;
    select('li').forEach(item => {
        const isSite = item.querySelector('.site')?.textContent === siteString;
        item.style.display = isSite ? '' : 'none';
    });

    select('.site').forEach(site => site.style.fontWeight = 'bold');
}

// Navigate to a version of current page with new params.
// Use pushState to not trigger a page reload and keep things responsive.
function apply(params) {
    const currentUrl = window.location.href.split('#')[0].split('?')[0];
    const newUrl = params.size ? `${currentUrl}?${params.toString()}${window.location.hash}` : `${currentUrl}${window.location.hash}`;
    window.history.pushState({}, "", newUrl);
    processParams()
}

function toggleTag(tag) {
    const urlParams = new URLSearchParams(window.location.search);
    const currentTags = urlParams.get("tags")?.split(' ') || [];

    if (currentTags.includes(tag)) {
        // Remove tag.
        const newTags = currentTags.filter(t => t !== tag);
        if (newTags.length) {
            urlParams.set("tags", newTags.join(" "));
        } else {
            urlParams.delete("tags");
        }
    } else {
        // Remove supersets of new tag from selection, having them
        // selected does not change the view.

        // Also remove subsets of the new tag, if the user is selecting a
        // superset tag, having a subset selected would not change the
        // current view, so assume the user wants it gone.
        let newTags = currentTags.filter(t =>
            !isSuperset(t, tag) && !isSuperset(tag, t) && !isDisjoint(tag, t));
        newTags.push(tag);
        urlParams.set("tags", newTags.join(" "));
    }

    apply(urlParams);
}

function toggleSite(site) {
    let urlParams = new URLSearchParams(window.location.search);
    if (urlParams.get("site") === site) {
        urlParams.delete("site");
    } else {
        urlParams.set("site", site);
    }
    apply(urlParams);
}

export function processParams() {
    const urlParams = new URLSearchParams(window.location.search);
    resetSelection();
    if (urlParams.has("tags")) {
        filterByTag(urlParams.get("tags").split(" "));
    }
    if (urlParams.has("site")) {
        filterBySite(urlParams.get("site"));
    }

    // If we're at root level (no selection params), grey out the nav bar link
    // for this page.
    const anchorElement = document.getElementById('banner-here');
    if (anchorElement) {
        if (!window.location.search) {
            anchorElement.className = "inactive";
        } else {
            anchorElement.className = "";
        }
    }
}

export function clickify() {
    select('a.site').forEach(link => {
        const site = link.textContent.slice(1, -1);
        link.onclick = function(event) {
            event.preventDefault();
            toggleSite(site);
        };
    });

    select('a.tag').forEach(link => {
        const tag = link.textContent;
        link.onclick = function(event) {
            event.preventDefault();
            toggleTag(tag);
        };
    });
}

