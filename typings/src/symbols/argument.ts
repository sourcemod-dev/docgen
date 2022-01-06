export interface Argument {
    /**
     * @brief Type of the argument
     * @readonly
     */
    readonly type: string,

    /**
     * @brief Name of the argument
     * @readonly
     */
    readonly name: string,

    /**
     * @brief Raw declaration of the argument
     * @readonly
     */
    readonly decl: string,

    /**
     * @brief Default value if any
     * @readonly
     */
    readonly default: string | null,
}
