# wayland-clipboard-manager  
A clipboard manager that keeps copied text alive even after the source application is closed.  

## TL;DR  
While developing my application [*Sherlock*](https://github.com/Skxxtz/sherlock), I wanted to implement a feature allowing users to calculate simple mathematical equations and store the results in the system clipboard. However, after closing the application, the result was no longer pasteable.  

After some research, I discovered that this was *intended behavior* and that I should "use a clipboard manager."  

Frustrated by this limitation, I followed the advice and looked for clipboard managers. I tried *wl-clipboard*, *cliphist*, *clipcat*, and others referenced in the Arch Wiki. However, I couldn't get them to work the way I wanted (...possibly due to my immense impatience).  

Long story short: I built this clipboard manager tailored to my needs. I still use *wl-clipboard* and *cliphist* for clipboard history, but now I also have a truly persistent clipboard.  

My implementation may not be perfect, as I had never worked with the Wayland client before, and resources on this topic were limited. One especially useful resource was @Decodetalkers' [wayland-clipboard-listener](https://github.com/Decodetalkers/wayland-clipboard-listener). Big shoutout to them for that!  

