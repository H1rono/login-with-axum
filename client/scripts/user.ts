type UserJSON = {
    id: string;
    display_id: string;
    name: string;
};

function isUserJSON(json: unknown): json is UserJSON {
    if (typeof json !== "object" || json === null) {
        return false;
    }
    const obj = json as Record<string, unknown>;
    const id = obj["id"];
    const displayId = obj["display_id"];
    const name = obj["name"];
    return typeof id === "string" && typeof displayId === "string" && typeof name === "string";
}

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
    constructor(id: string, displayId: string, name: string) {
        this.#_id = id;
        this.#_displayId = displayId;
        this.#_name = name;
    }

    static fromJSON(json: unknown): User | undefined {
        if (!isUserJSON(json)) {
            console.error("Invalid JSON data");
            return undefined;
        }
        const id = json["id"];
        const displayId = json["display_id"];
        const name = json["name"];

        return new User(id, displayId, name);
    }

    get id(): string {
        return this.#_id;
    }

    get displayId(): string {
        return this.#_displayId;
    }

    get name(): string {
        return this.#_name;
    }
}

export default User;
