import { symbolDistance } from '@/utils';
import { Comment } from './dcp';
import { SymbolYield, Containment } from './search';
import { Property } from './property';
import { Field } from './enum_struct';

/**
 * Generator data
 */
export interface Documentation {
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
