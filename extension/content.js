document.addEventListener('copy', () => {
    const context = {
        url: window.location.href,
        title: document.title
    };

    fetch('http://127.0.0.1:17531/context', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(context)
    }).catch((err) => {
        // Silently fail if the Mnemo backend is not running
        console.debug('Mnemo context bridge failed to connect:', err);
    });
});
