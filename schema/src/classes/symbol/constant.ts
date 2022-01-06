import { IConstant } from '../../interfaces';
import { Declaration } from './base';

export class Constant extends Declaration implements IConstant {
    public constructor(constant: IConstant) {
        super(constant);
    }
}
