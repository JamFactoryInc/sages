const parseDate = str => {
    let d = Date.parse(str);
    return isNaN(d) ? Date.now() : d;
};

addEventListener("DOMContentLoaded", e => {
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

    const MS_IN_YEAR = 1000 * 60 * 60 * 24 * 365;
    let timeLineDuration = timelineEnd - timelineStart;
    let thisYear = new Date().getFullYear();
    let firstDayThisYear = new Date(thisYear, 0).getTime();

    spans = spans.map(span => {
        return {
            element: span.element,
            width_pct: (span.end - span.start) / timeLineDuration,
            offset_pct: (span.start - timelineStart) / timeLineDuration
        };
    });

    let years = [];
    let timelineElement = document.querySelector("#timeline");

    for (let yearStart = firstDayThisYear; yearStart > timelineStart; yearStart -= MS_IN_YEAR) {
        let element = document.createElement("span");
        element.setAttribute("year", thisYear--);
        element.className = "timeline-year";
        timelineElement.appendChild(element);
        years.push({
            element: element,
            offset_pct: (yearStart - timelineStart) / timeLineDuration,
        })
    }
    [{
        text: new Date(timelineStart).toLocaleString("default", { month: "short", year: "numeric" }),
        offset: 0
    }, {
            text: "Today",
            offset: 1
        }].forEach(year => {
            let element = document.createElement("span");
            element.setAttribute("year", year.text);
            element.className = "timeline-year";
            timelineElement.appendChild(element);
            years.push({
                element: element,
                offset_pct: year.offset,
            });
        });

    let onResize = e => {
        let computedStyle = getComputedStyle(timeLine);
        let timeLineWidth = timeLine.clientWidth - parseFloat(computedStyle.paddingLeft) - parseFloat(computedStyle.paddingRight);
        for (let i = 0; i < spans.length; i++) {
            let span = spans[i];
            span.element.style["margin-left"] = (timeLineWidth * span.offset_pct) + "px";
            span.element.style["width"] = (timeLineWidth * span.width_pct) + "px";
        }
        for (let i = 0; i < years.length; i++) {
            let year = years[i];
            year.element.style["margin-left"] = (timeLineWidth * year.offset_pct) + "px";
        }
    };

    addEventListener("resize", onResize);

    onResize();
});