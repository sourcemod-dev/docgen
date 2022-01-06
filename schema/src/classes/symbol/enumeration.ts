import { Declaration } from './base';
import { IEnumeration, IEntry, Searchable } from '../../interfaces';

export class Enumeration extends Declaration implements IEnumeration, Searchable {
    /**
     * @brief Enum entries
     * @readonly
     */
     readonly entries: IEntry[];

    public constructor(enumeration: IEnumeration) {
        super(enumeration);

        this.entries = enumeration.entries;
    }
}
