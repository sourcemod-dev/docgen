export interface Tag {
    /**
     * @brief Tag name
     * @readonly
     */
    readonly tag: string,

    /**
     * @brief Tag textual content
     * @readonly
     */
    readonly text: string,
}

export interface Comment {
    /**
     * @brief Brief description of the function's purpose
     * @readonly
     */
    readonly brief: string,

    /**
     * @brief Tags of the symbol
     * @readonly
     */
    readonly tags: Tag[],
}
