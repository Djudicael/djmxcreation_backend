export function formatDate(dateToFormat) {
    dateToFormat = new Date(dateToFormat);
    const monthNames = [
        "January", "February", "March",
        "April", "May", "June", "July",
        "August", "September", "October",
        "November", "December"
    ];
    const days = [
        "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"
    ];
    const date = dateToFormat.getDate();
    const monthIndex = dateToFormat.getMonth();
    const year = dateToFormat.getFullYear();
    const day = dateToFormat.getDay();
    return `${days[day]} ${monthNames[monthIndex]} ${date} ${year}`;
}

export function formatDateObject(dateToFormat) {
    dateToFormat = new Date(dateToFormat);
    const monthNames = [
        "January", "February", "March",
        "April", "May", "June", "July",
        "August", "September", "October",
        "November", "December"
    ];
    const days = [
        "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"
    ];
    const date = dateToFormat.getDate();
    const monthIndex = dateToFormat.getMonth();
    const year = dateToFormat.getFullYear();
    const day = dateToFormat.getDay();
    return {
        day: days[day],
        monthName: monthNames[monthIndex],
        year,
        date
    }
}