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
export function rootPath(relative: string): string | undefined {
    let tail = relative;
    if (tail.startsWith("./")) {
        tail = tail.substring(1);
    }
    if (!tail.startsWith("/")) {
        tail = "/" + tail;
    }

    const pathname = globalThis.location.pathname;
    if (!pathname.endsWith(tail)) {
        return undefined;
    }
    return pathname.substring(0, pathname.length - tail.length);
}
