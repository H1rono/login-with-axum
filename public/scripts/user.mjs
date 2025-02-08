class User {
    #_id;
    #_displayId;
    #_name;

    /**
     * Constructor
     * @param {String} id UUID
     * @param {String} displayId Display ID
     * @param {String} name User name
     */
    constructor(id, displayId, name) {
        this.#_id = id;
        this.#_displayId = displayId;
        this.#_name = name;
    }

    static fromJSON(json) {
        const id = json["id"];
        const displayId = json["display_id"];
        const name = json["name"];

        const validId = typeof id === 'string';
        const validDisplayId = typeof displayId === 'string';
        const validName = typeof name === 'string';
        const valid = validId && validDisplayId && validName;
        if (!valid) {
            console.error('Invalid JSON data');
            return undefined;
        }

        return new User(id, displayId, name);
    }

    /**
     * Get the UUID
     * @returns {String} UUID
     */
    get id() {
        return this.#_id;
    }

    /**
     * Get the display ID
     * @returns {String} Display ID
     */
    get displayId() {
        return this.#_displayId;
    }

    /**
     * Get the user name
     * @returns {String} User name
     */
    get name() {
        return this.#_name;
    }
}

export default User;
