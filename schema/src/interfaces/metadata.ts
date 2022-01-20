export interface Metadata {
    /**
     * SVN version this symbol was introduced
     */
    created: IVersioning | null;

    /**
     * SVN version this symbol was last modified
     */
    last_updated: IVersioning | null;
}

export interface IVersioning {
    hash: string;

    /**
     * Rev-list count
     * Mainly used for core where product.version will be within spec paths
     */
    count: number;

    /**
     * Unix timestamp, 64 bit, doesn't fit in JS number
     */
    time: string;
}
