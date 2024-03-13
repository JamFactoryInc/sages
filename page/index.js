const parseDate = str => {
    let d = Date.parse(str);
    return isNaN(d) ? Date.now() : d;
}

addEventListener("load", e => {
    let timeLine = document.getElementById('timeline');
    let timelineBlocks = [...document.getElementsByClassName('timeline-block')];

    let timelineStart = Number.MAX_SAFE_INTEGER;
    let timelineEnd = Date.now();

    let spans = timelineBlocks.map(experience => {
        let span = {
            element: experience,
            start: parseDate(experience.getAttribute("from")),
            end: parseDate(experience.getAttribute("to")),
        };
        if (span.start < timelineStart) {
            timelineStart = span.start;
        }
        return span;
    });

    let timeLineDuration = timelineEnd - timelineStart;

    spans = spans.map(span => {
        return {
            element: span.element,
            width_pct: (span.end - span.start) / timeLineDuration,
            offset_pct: (span.start - timelineStart) / timeLineDuration
        }
    });

    let onResize = e => {
        let computedStyle = getComputedStyle(timeLine);
        let timeLineWidth = timeLine.clientWidth - parseFloat(computedStyle.paddingLeft) - parseFloat(computedStyle.paddingRight);
        for (let i = 0; i < spans.length; i++) {
            let span = spans[i];
            span.element.style["margin-left"] = (timeLineWidth * span.offset_pct) + "px";
            span.element.style["width"] = (timeLineWidth * span.width_pct) + "px";
        }
    }

    addEventListener("resize", onResize);

    onResize()
});