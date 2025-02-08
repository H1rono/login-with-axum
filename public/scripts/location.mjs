/**
 * Get the root path of the current location, triming the last part of the path.
 * Returns `undefined` if the relative path is not tail of the current location
 * @param {String} relative relative path of current location
 * @returns {String} root path of the current location
 * @example
 * // assuming the current location is '/root/scripts/location.mjs'
 * rootPath('scripts/location.mjs') // => '/root'
 * rootPath('scripts') // => undefined
 */
export function rootPath(relative) {
    var tail = relative;
    if (tail.startsWith('./')) {
        tail = tail.substring(1);
    }
    if (!tail.startsWith('/')) {
        tail = '/' + tail;
    }

    if (!window.location.pathname.endsWith(tail)) {
        return undefined;
    }
    return window.location.pathname.substring(0, window.location.pathname.length - tail.length);
}
