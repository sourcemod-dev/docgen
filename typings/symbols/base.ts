import { Comment } from './dcp';

/**
 * Generator data
 */
export interface Documentation {
    /**
     * @brief Symbol reference line number
     * @readonly
     */
    readonly refLine: number,

    /**
     * @brief Documentation starting byte
     * @readonly
     */
    readonly docStart: number,

    /**
     * @brief Documentation ending byte
     * @readonly
     */
    readonly docEnd: number,

    /**
     * @brief Parsed documentation
     * @note Null if docStart or docEnd is 0
     * @readonly
     */
    readonly docs: Comment | null,
}

/**
 * @brief Base symbol declaration
 */
export interface Declaration extends Documentation {
    readonly name: string,
}
