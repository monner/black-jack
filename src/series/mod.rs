//! Series represents a single column within a dataframe and wraps many `Array` like
//! functionality.
//! 
//! For methods implemented for a [`Series`], please check out the trait [`SeriesTrait`]
//! 
//! ## Example use:
//! 
//! ```
//! use blackjack::prelude::*;
//! 
//! let mut series = Series::arange(0, 5);
//! 
//! // Index and change elements, call `.into()` to easily convert to `DataElement`
//! series[0] = 1.into();              // `into()` on `BlackJackData`
//! series[1] = DataElement::I32(0);   // ...or more explicitly set the value
//! 
//! assert_eq!(series[0], DataElement::I32(1));
//! assert_eq!(series.sum::<i32>(), 10);
//! assert_eq!(series.len(), 5);
//! ```

use std::ops::{Range, Index, IndexMut};
use std::iter::{FromIterator, Sum};
use std::convert::From;
use std::fmt;

use num::*;
use stats;

pub mod overloaders;
use prelude::*;


/// Series struct for containing underlying Array and other meta data.
#[derive(Debug, Clone, PartialEq)]
pub struct Series {
    
    /// Name of the series, if added to a dataframe without a name, it will be assigned
    /// a default name equalling the cound of columns in the dataframe.
    pub name: Option<String>,

    /// The underlying values of the Series
    pub values: Vec<DataElement>,

    // Only set if called by `.astype()` or parsing or raw data was able to
    // confirm all `DataElement`s are of the same type.
    dtype: Option<DType>
}

impl Index<usize> for Series {
    type Output = DataElement;
    fn index(&self, idx: usize) -> &DataElement {
        &self.values[idx]
    }
}

impl IndexMut<usize> for Series {
    fn index_mut(&mut self, idx: usize) -> &mut DataElement {
        &mut self.values[idx]
    }
}

impl fmt::Display for Series {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        let mut string = "".to_string();
        let name = self.name().unwrap_or("None".to_string());
        
        // Write name inside column
        let header = format!("| {} |\n", &name);
        string.push_str(&header);

        // Start writing rows... 
        for val in &self.values {

            let mut row_string = "|".to_string();
            let val: String = val.clone().into();

            while row_string.len() < (header.len() / 2) - val.len() as usize {
                row_string.push_str(" ");
            }

            row_string.push_str(&val);
            
            while row_string.len() < header.len() - 2 {
                row_string.push_str(" ");
            }

            row_string.push_str("|\n");
            string.push_str(&row_string);
        }

        write!(f, "{}\n", string)
    }
}

/// Constructor methods for `Series<T>`
impl Series {

    /// Create a new Series struct from an integer range with one step increments. 
    /// 
    /// ## Example
    /// ```
    /// use blackjack::prelude::*;
    /// 
    /// let series: Series = Series::arange(0, 10);
    /// ```
    pub fn arange<T>(start: T, stop: T) -> Self 
        where
            T: Integer + BlackJackData + ToPrimitive,
            Range<T>: Iterator, 
            Vec<T>: FromIterator<<Range<T> as Iterator>::Item>
    {
        let dtype = Some(start.dtype());
        let data: Vec<T> = (start..stop).collect();
        let vec: Vec<DataElement> = data.into_iter().map(|v| DataElement::from(v)).collect();
        Series { 
            name: None,
            dtype,
            values: vec, 
        }
    }

    /// Create a new Series struct from a vector, where T is supported by [`BlackJackData`]. 
    /// 
    /// ## Example
    /// ```
    /// use blackjack::prelude::*;
    /// 
    /// let series: Series = Series::from_vec(vec![1, 2, 3]);
    /// ```
    pub fn from_vec<T>(vec: Vec<T>) -> Self 
        where 
            T: BlackJackData,
            DataElement: From<T>
    {
        let dtype = if vec.len() > 0 { Some(vec[0].dtype()) } else  { None };
        let vec: Vec<DataElement> = vec.into_iter().map(|v| DataElement::from(v)).collect();
        Series { 
            name: None,
            dtype,
            values: vec,
        }
    }

    /// Create series from a vector of [`DataElement`] enums. 
    /// Useful in constructing a [`Vec`] from various data types.
    /// ## Example
    /// ```
    /// use blackjack::prelude::*;
    /// 
    /// let series = Series::from_data_elements(vec![
    ///     DataElement::F64(1.0),
    ///     DataElement::I32(2),
    ///     DataElement::STRING("Hello there".to_string())
    /// ]);
    /// 
    /// assert_eq!(series.len(), 3);
    /// assert_eq!(series.dtype(), None); // DType is unknown, use `.astype()` for coercion
    /// ```
    pub fn from_data_elements(vec: Vec<DataElement>) -> Self {

        // TODO: Add check to see if all DataElements are of the same dtype.
        Series {
            name: None,
            dtype: None,
            values: vec,
        }
    }

    /// Convert the series to a [`Vec`]  
    /// **Type Annotations required**
    /// Will coerce elements into the desired [`DType`] primitive, just as
    /// [`SeriesTrait::astype()`]. 
    /// 
    /// ## Example
    /// ```
    /// use blackjack::prelude::*;
    /// 
    /// let series = Series::from_vec(vec![1_f64, 2_f64, 3_f64]);
    /// 
    /// assert_eq!(
    ///     series.clone().to_vec::<i32>(), 
    ///     vec![1_i32, 2_i32, 3_i32]
    /// );
    /// assert_eq!(
    ///     series.to_vec::<String>(), 
    ///     vec![1_f64.to_string(), 2_f64.to_string(), 3_f64.to_string()]
    /// );
    /// ```
    pub fn to_vec<T: From<DataElement>>(self) -> Vec<T> {
        let vec: Vec<T> = self.values.into_iter().map(|v| T::from(v.clone())).collect();
        vec
    }

    /// Set the name of a series
    pub fn set_name(&mut self, name: &str) -> () {
        self.name = Some(name.to_string());
    }

    /// Get the name of the series; Series may not be assigned a string, 
    /// so an `Option` is returned.
    /// 
    /// ## Example
    /// ```
    /// use blackjack::prelude::*;
    /// 
    /// let mut series = Series::from_vec(vec![1, 2, 3]);
    /// series.set_name("my-series");
    /// 
    /// assert_eq!(series.name(), Some("my-series".to_string()));
    /// ```
    pub fn name(&self) -> Option<String> {
        match self.name {
            Some(ref name) => Some(name.clone()),
            None => None
        }
    }

    /// Finds the returns a [`Series`] containing the mode(s) of the current
    /// [`Series`]
    pub fn mode<T>(&self) -> Result<Self, &'static str> 
        where T: BlackJackData + From<DataElement> + PartialOrd + Clone + ToPrimitive
    {
        if self.len() == 0 {
            return Err("Cannot compute mode of an empty series!")
        }

        let modes = stats::modes(self.values.iter().map(|v| T::from(v.clone())));
        let mut modes = Series::from_vec(modes);

        // Cast to the requested DType 'T'
        modes.astype(T::from(self.values[0].clone()).dtype())?;
        Ok(modes)
    }

    /// Calculate the variance of the series  
    /// **NOTE** that whatever type is determined is what the values are cast to
    /// during calculation of the variance. 
    /// 
    /// ie. `series.var::<i32>()` will cast each element into `i32` as input
    /// for calculating the variance, and yield a `i32` value. If you want all
    /// values to be calculated as `f64` then specify that in the type annotation.
    pub fn var<T>(&self) -> Result<T, &'static str>
        where 
            T: BlackJackData + From<DataElement> + ToPrimitive + Clone
    {
        if self.len() == 0  {
            return Err("Cannot compute variance of an empty series!");
        }
        let var = stats::variance(self.values.iter().map(|v| T::from(v.clone())));
        Ok(DataElement::from(var).into())
    }

    /// Sum a given series, yielding the same type as the elements stored in the 
    /// series.
    pub fn sum<T>(&self) -> T
        where 
            T: Num + Clone + From<DataElement> + Sum + Copy
    {
        self.values.iter()
            .filter(|v| v.dtype() != DType::STRING)  // No strings allowed
            .filter(|v| !v.is_nan())                 // or NaNs
            .map(|v| T::from(v.clone()))
            .sum()
    }

    /// Average / Mean of a given series - Requires specifying desired float 
    /// return annotation 
    /// 
    /// ## Example:
    /// ```
    /// use blackjack::prelude::*;
    /// 
    /// let series = Series::arange(0, 5);
    /// let mean = series.mean();
    /// 
    /// match mean {
    ///     Ok(result) => {
    ///         println!("Result is: {}", &result);
    ///         assert_eq!(result, 2.0);
    ///     },
    ///     Err(err) => {
    ///         panic!("Was unable to compute mean, error: {}", err);
    ///     }
    /// }
    /// ```
    pub fn mean(&self) -> Result<f64, &'static str>
    {
        let total: f64 = self.sum();
        let count: f64 = self.len() as f64;
        Ok(total / count)
    }

    /// Find the minimum of the series. If several elements are equally minimum,
    /// the first element is returned. If it's empty, an Error will be returned.
    /// 
    /// ## Example
    /// ```
    /// use blackjack::prelude::*;
    /// 
    /// let series: Series = Series::arange(10, 100);
    /// 
    /// assert_eq!(series.min(), Ok(10));
    /// ```
    pub fn min<T>(&self) -> Result<T, &'static str>
        where 
            T: Num + Clone + Ord + BlackJackData + From<DataElement>
    {
        let min = self.values.iter().map(|v| T::from(v.clone())).min();
        match min {
            Some(m) => Ok(m),
            None => Err("Unable to find minimum of values, perhaps values is empty?")
        }
    }

    /// Exibits the same behavior and usage of [`SeriesTrait::min`], only
    /// yielding the [`Result`] of a maximum.
    pub fn max<T>(&self) -> Result<T, &'static str>
        where 
            T: Num + Clone + Ord,
            T: From<DataElement>
    {
        let max = self.values.iter().map(|v| T::from(v.clone())).max();
        match max {
            Some(m) => Ok(m),
            None => Err("Unable to find maximum of values, perhaps values is empty?")
        }
    }

    /// Determine the length of the Series
    pub fn len(&self) -> usize { self.values.len() }

    /// Determine if series is empty.
    pub fn is_empty(&self) -> bool { self.len() == 0 }

    /// Get the dtype, returns `None` if series dtype is unknown. 
    /// in such a case, calling `.astype()` to coerce all types to a single
    /// type is needed. 
    pub fn dtype(&self) -> Option<DType> { 
        self.dtype.clone()
    }

    /// Cast all [`DataElement`]s within a series to a given [`DType`]
    /// Will fail if series contains a string and asking for an integer, 
    /// of an `NaN` and asking for an integer.
    /// 
    /// ie. "Hello" -> .astype([`DType::I64`]) -> **Error!**  
    /// ie. "Hello" -> .astype([`DType::F64`]) -> `NaN`  
    /// ipso-facto... `NaN` -> .astype([`DType::I64`]) -> **Error!**
    pub fn astype(&mut self, dtype: DType) -> Result<(), &'static str> {
    
        // iterate over all elements currently held...
        for val in &mut self.values {

            // Convert the value to the desired dtype
            *val = match dtype {
                DType::F64 => DataElement::F64(val.into()),
                DType::I64 => {
                    if val.dtype() == DType::STRING || val.is_nan() {
                        return Err("Cannot convert Float NaN to Integer type")
                    } else {
                        DataElement::I64(val.into())
                    }
                }
                DType::F32 => DataElement::F32(val.into()),
                DType::I32 => {
                    if val.dtype() == DType::STRING || val.is_nan() {
                        return Err("Cannot convert Float NaN to Integer type")
                    } else {
                        DataElement::I32(val.into())
                    }
                },
                DType::STRING => DataElement::STRING(val.into()),
                DType::None => DataElement::None
            }
        };

         // Now all elements are converted, set `dtype`
        self.dtype = Some(dtype);

        Ok(())
    }

    /// Append a [`BlackJackData`] element to the Series
    /// 
    /// ## Example
    /// ```
    /// use blackjack::prelude::*;
    /// 
    /// let mut series = Series::from_vec(vec![0, 1, 2]);
    /// assert_eq!(series.len(), 3);
    /// 
    /// series.append(3);
    /// assert_eq!(series.len(), 4);
    /// ```
    pub fn append<V: Into<DataElement>>(&mut self, val: V) -> () {
        self.values.push(val.into());
    }

    /// As boxed pointer, recoverable by `Box::from_raw(ptr)` or 
    /// `SeriesTrait::from_raw(*mut Self)`
    pub fn into_raw(self) -> *mut Self { 
        Box::into_raw(Box::new(self)) 
    }

    /// Create from raw pointer
    pub fn from_raw(ptr: *mut Self) -> Self { 
        unsafe { *Box::from_raw(ptr) } 
    }
}
